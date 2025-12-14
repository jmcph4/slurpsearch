use eyre::WrapErr;
use futures::{StreamExt, stream};
use std::{sync::Arc, time::Instant};
use tokio::task::LocalSet;
use tokio::time::{Duration, timeout};
use tracing::{debug, warn};
use url::Url;

use playwright::api::{Browser, BrowserContext, Page, Playwright};

/// Small helper so we can log scheme/host/path without dumping full URL (which may include secrets).
fn url_brief(url: &Url) -> String {
    let host = url.host_str().unwrap_or("<no-host>");
    format!("{}://{}{}", url.scheme(), host, url.path())
}

pub struct HtmlFetcher {
    // Keep these alive: dropping Browser/Playwright can invalidate the context/page and make all
    // fetches fail.
    _playwright: Playwright,
    _browser: Browser,
    context: BrowserContext,
}

impl HtmlFetcher {
    #[tracing::instrument(level = "debug", name = "html_fetcher.new", skip_all)]
    pub async fn new() -> eyre::Result<Self> {
        let playwright = Playwright::initialize()
            .await
            .wrap_err("playwright initialize failed")?;

        // playwright 0.0.20: chromium() returns BrowserType directly
        let chromium = playwright.chromium();

        let browser = chromium
            .launcher()
            .headless(true)
            .launch()
            .await
            .wrap_err("chromium launch failed")?;

        let context = browser
            .context_builder()
            .build()
            .await
            .wrap_err("browser context build failed")?;

        debug!("playwright chromium launched + context created");

        Ok(Self {
            _playwright: playwright,
            _browser: browser,
            context,
        })
    }

    async fn new_page(&self) -> eyre::Result<Page> {
        self.context
            .new_page()
            .await
            .wrap_err("context.new_page failed")
    }

    #[tracing::instrument(
        level = "debug",
        name = "html_fetcher.fetch_html",
        skip_all,
        fields(url = %url)
    )]
    pub async fn fetch_html(&self, url: Url) -> eyre::Result<String> {
        let started = Instant::now();
        debug!("fetch start: {}", url_brief(&url));

        let page = self.new_page().await?;

        // Try a "wait until network idle" navigation first.
        let goto_res = page
            .goto_builder(url.as_str())
            .wait_until(playwright::api::DocumentLoadState::NetworkIdle)
            .goto()
            .await;

        if let Err(e) = goto_res {
            warn!(
                "goto(NetworkIdle) failed for {}: {:?}; retrying with Load",
                url_brief(&url),
                e
            );

            page.goto_builder(url.as_str())
                .wait_until(playwright::api::DocumentLoadState::Load)
                .goto()
                .await
                .wrap_err("goto(Load) failed")?;
        }

        // Give client-side apps a moment to paint. Non-fatal if it fails.
        if let Err(e) = page
            .eval::<bool>(
                "() => new Promise(r => setTimeout(() => r(true), 250))",
            )
            .await
        {
            debug!(
                "post-load settle eval failed for {}: {:?} (continuing)",
                url_brief(&url),
                e
            );
        }

        let html = page.content().await.wrap_err("page.content failed")?;

        debug!(
            "fetch ok: {} bytes={} elapsed_ms={}",
            url_brief(&url),
            html.len(),
            started.elapsed().as_millis()
        );

        if let Err(e) = page.close(None).await {
            debug!("page.close failed for {}: {:?}", url_brief(&url), e);
        }

        Ok(html)
    }
}

/// Fetch all HTML in parallel, returning a Vec of (Url, Result<html>).
///
/// This runs the entire Playwright pipeline on a single Tokio thread (via `LocalSet`) to avoid
/// "Object not found" failures that can happen if Playwright objects are used across threads.
#[tracing::instrument(level = "debug", name = "fetch_all_html", skip_all, fields(concurrency = concurrency))]
pub async fn fetch_all_html<I>(
    urls: I,
    concurrency: usize,
) -> eyre::Result<Vec<(Url, eyre::Result<String>)>>
where
    I: IntoIterator<Item = Url>,
{
    let urls: Vec<Url> = urls.into_iter().collect();

    let local = LocalSet::new();
    local
        .run_until(async move {
            let fetcher = Arc::new(HtmlFetcher::new().await?);

            let total = urls.len();
            debug!("bulk fetch start: total_urls={total} concurrency={concurrency}");

            let started = Instant::now();
            let mut ok = 0usize;
            let mut err = 0usize;
            let mut done = 0usize;

            let per_url_timeout = Duration::from_secs(45);

            let stream = stream::iter(urls.into_iter().map(|url| {
                let fetcher = Arc::clone(&fetcher);
                async move {
                    let u = url.clone();
                    let brief = url_brief(&u);

                    let res = match timeout(per_url_timeout, fetcher.fetch_html(url.clone())).await
                    {
                        Ok(r) => r,
                        Err(_) => {
                            warn!("timeout fetching {}", brief);
                            Err(eyre::eyre!("timeout after {:?}", per_url_timeout))
                        }
                    };

                    (u, res)
                }
            }))
            .buffer_unordered(concurrency);

            let mut out: Vec<(Url, eyre::Result<String>)> = Vec::with_capacity(total);

            tokio::pin!(stream);
            while let Some((url, res)) = stream.next().await {
                done += 1;
                match &res {
                    Ok(_) => ok += 1,
                    Err(e) => {
                        err += 1;
                        debug!("fetch failed: {} err={:?}", url_brief(&url), e);
                    }
                }

                if done.is_multiple_of(100) || done == total {
                    debug!(
                        "bulk fetch progress: done={done}/{total} ok={ok} err={err} elapsed_s={}",
                        started.elapsed().as_secs()
                    );
                }

                out.push((url, res));
            }

            debug!(
                "bulk fetch complete: total={total} ok={ok} err={err} elapsed_s={}",
                started.elapsed().as_secs()
            );

            Ok(out)
        })
        .await
}
