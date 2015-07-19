# Gitostat

Tool for obtaining different kind of information from your git repository.

### Current functionals
* Heatmap of the most active hours of the week

    ```
           0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20 21 22 23
    Mon:   .  .  .  .  .  .  .  .  .  .  ◾  .  .  .  .  .  .  .  .  .  .  ▪  .  .
    Tue:   ▪  .  .  .  .  .  .  .  .  .  .  ▪  .  .  .  .  .  .  .  .  .  .  .  .
    Wed:   ◾  .  .  .  .  .  .  .  .  ▪  .  .  .  .  .  .  .  .  .  .  ▪  .  .  .
    Thu:   .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  ▪  .  ▪
    Fri:   .  .  .  .  .  .  .  .  .  .  .  .  .  ▪  .  .  ▪  ◾  .  .  ▪  .  ▪  .
    Sat:   .  .  .  .  .  .  .  .  .  .  .  .  .  ▪  ⬛  .  .  .  .  .  .  .  ▪  .
    Sun:   ▪  .  .  .  .  .  .  .  .  .  ▪  .  .  .  .  .  ▪  .  .  .  .  ▪  ▪  ◾
    ```

* Files snapshot with counting relative amount of the file types

    ```
    .gitignore
    .mailmap
    Cargo.lock
    Cargo.toml
    LICENSE
    README.md
    src/heatmap.rs
    src/languages.yml
    src/mailmap.rs
    src/main.rs
    src/personal.rs
    src/snapshot.rs
    Total files: 12
     2015-07-19 16:04:12 +06:00
    ░░░░░░░▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓███████░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
    ["lock", "md", "rs", "toml", "yml", "other"]
    ```

* Counting personal stats of authors (with respect .mailmap)

    ```
    +-------------------------------------------+--------------+------------+-----------+-----+-------------+
    | Author                                    | Commits (%)  | Insertions | Deletions | Age | Active days |
    +-------------------------------------------+--------------+------------+-----------+-----+-------------+
    | Arthur Skobara <skobara.arthur@gmail.com> | 29 (100.00%) | 5762       | 1165      | 78  | 9           |
    +-------------------------------------------+--------------+------------+-----------+-----+-------------+
    | Total                                     | 29 (100%)    | 5762       | 1165      | 78  | 9           |
    +-------------------------------------------+--------------+------------+-----------+-----+-------------+
    ```

### TODO
* Counting average age of the repo
* Counting percent of author's code
