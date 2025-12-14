# slurpsearch #

Given a file containing URLs, extract each one and search the resource located there (if it exists) for the provided search term.

## Usage ##

```
$ slurpsearch -h
Usage: slurpsearch <HAYSTACK> <NEEDLE>

Arguments:
  <HAYSTACK>  Path to file to search
  <NEEDLE>    Search term

Options:
  -h, --help  Print help
```

For example,

```
$ cat test.md

 - [2025-12-12T17:18:12+10:00] [Scatterpad](https://scatterpad.com)

---

 - [2025-12-12T17:18:12+10:00] [Streams](https://streams.place)

---

 - [2025-12-12T22:22:20+10:00] [My productivity app is a never-ending .txt file](https://jeffhuang.com/productivity_text_file)

---

 - [2025-12-12T22:22:20+10:00] [Jeff Huang - Computer Science at Brown University](https://jeffhuang.com)

---

 - [2025-12-13T19:00:33+10:00] [Rose Yu Homepage](https://roseyu.com)

---

 - [2025-12-14T13:58:12+10:00] [All posts | Faith's Blog](https://faith2dxy.xyz)

---

 - [2025-12-14T14:15:10+10:00] [Blog](https://u1f383.github.io)

---
$ slurpsearch test.md file
2025-12-14T06:20:43.384494Z  INFO slurpsearch: Extracted 7 URLs from test.md
2025-12-14T06:20:43.384549Z  INFO slurpsearch: Retrieving HTML...
2025-12-14T06:20:50.552868Z  INFO slurpsearch: Retrieved 7 webpages
2025-12-14T06:20:50.552927Z  INFO slurpsearch: Commencing full-text search...
Found hit for "file" in https://faith2dxy.xyz/ on line 1 column 667
Found hit for "file" in https://faith2dxy.xyz/ on line 1 column 897
Found hit for "file" in https://faith2dxy.xyz/ on line 1 column 1317
Found hit for "file" in https://faith2dxy.xyz/ on line 3 column 1707
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 710
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 778
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 895
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 963
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 1095
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 1164
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 1236
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 1303
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 1435
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 1503
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 1621
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 1689
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 1905
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 1977
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 2044
Found hit for "file" in https://faith2dxy.xyz/ on line 57 column 2071
Found hit for "file" in https://jeffhuang.com/productivity_text_file on line 3 column 51
Found hit for "file" in https://jeffhuang.com/productivity_text_file on line 112 column 48
Found hit for "file" in https://jeffhuang.com/productivity_text_file on line 120 column 54
Found hit for "file" in https://jeffhuang.com/productivity_text_file on line 124 column 153
Found hit for "file" in https://jeffhuang.com/productivity_text_file on line 147 column 214
Found hit for "file" in https://jeffhuang.com/productivity_text_file on line 149 column 12
Found hit for "file" in https://jeffhuang.com/productivity_text_file on line 149 column 169
Found hit for "file" in https://jeffhuang.com/productivity_text_file on line 196 column 8
Found hit for "file" in https://jeffhuang.com/productivity_text_file on line 214 column 62
Found hit for "file" in https://jeffhuang.com/ on line 165 column 36
Found hit for "file" in https://jeffhuang.com/ on line 165 column 86
Found hit for "file" in https://streams.place/ on line 698 column 44
Found hit for "file" in https://roseyu.com/ on line 16 column 68
Found hit for "file" in https://roseyu.com/ on line 70 column 37
Found hit for "file" in https://roseyu.com/ on line 1093 column 61
Found hit for "file" in https://roseyu.com/ on line 1101 column 61
Found hit for "file" in https://roseyu.com/ on line 1128 column 59
Found hit for "file" in https://roseyu.com/ on line 1137 column 59
Found hit for "file" in https://u1f383.github.io/ on line 247 column 62
```

