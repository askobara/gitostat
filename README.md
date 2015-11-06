# Gitostat
[![Build Status](https://travis-ci.org/askobara/gitostat.svg?branch=master)](https://travis-ci.org/askobara/gitostat)

[![Build Status](https://codeship.com/projects/d5719a20-66c5-0133-ca70-3685ec7a2958/status?branch=master)](https://codeship.com/projects/113864)

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

* Files snapshot with counting relative amount of the file types which using in graphics below

    ```
    Files in repo:
    2014-10-05   5 ⚫⚫⚫⚫⚫
    2014-10-12   5 ⚫⚫⚫⚫⚫
    2014-10-14   5 ⚫⚫⚫⚫⚫
    2014-10-15   5 ⚫⚫⚫⚫⚫
    2015-04-03   5 ⚫⚫⚫⚫⚫
    2015-04-04   5 ⚫⚫⚫⚫⚫
    2015-04-05   5 ⚫⚫⚫⚫⚫
    2015-04-06   5 ⚫⚫⚫⚫⚫
    2015-04-07   5 ⚫⚫⚫⚫⚫
    2015-04-08   5 ⚫⚫⚫⚫⚫
    2015-05-01   7 ⚫⚫⚫⚫⚫⚫⚫
    2015-05-02   8 ⚫⚫⚫⚫⚫⚫⚫⚫
    2015-05-31   9 ⚫⚫⚫⚫⚫⚫⚫⚫⚫
    2015-06-14  11 ⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫
    2015-06-25  11 ⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫
    2015-06-28  12 ⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫
    2015-07-06  12 ⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫
    2015-07-09  12 ⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫
    2015-07-15  12 ⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫
    2015-07-19  12 ⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫
    2015-07-20  13 ⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫⚫
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

* Activity in repo by weeks

    ```
    Oct 2014   1 ⚫
    Oct 2014   1 ⚫
    Oct 2014   2 ⚫⚫
    Oct 2014   0
    Nov 2014   0
    Nov 2014   0
    Nov 2014   0
    Nov 2014   0
    Nov 2014   0
    Dec 2014   0
    Dec 2014   0
    Dec 2014   0
    Dec 2014   0
    Jan 2015   0
    Jan 2015   0
    Jan 2015   0
    Jan 2015   0
    Feb 2015   0
    Feb 2015   0
    Feb 2015   0
    Feb 2015   0
    Mar 2015   0
    Mar 2015   0
    Mar 2015   0
    Mar 2015   0
    Mar 2015   0
    Apr 2015   4 ⚫⚫⚫⚫
    Apr 2015   3 ⚫⚫⚫
    Apr 2015   0
    Apr 2015   0
    May 2015   8 ⚫⚫⚫⚫⚫⚫⚫⚫
    May 2015   0
    May 2015   0
    May 2015   0
    May 2015   1 ⚫
    Jun 2015   0
    Jun 2015   1 ⚫
    Jun 2015   0
    Jun 2015   2 ⚫⚫
    Jul 2015   0
    Jul 2015   3 ⚫⚫⚫
    Jul 2015   3 ⚫⚫⚫
    Jul 2015   1 ⚫
    Aug 2015   0
    ```

### TODO
* More tests
* Counting average age of the repo
* Counting percent of author's code
