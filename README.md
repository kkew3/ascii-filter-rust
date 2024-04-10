# ascii-filter

This utility filters out all bytes other than ASCII letters, digits, ASCII punctuations, space, tab, and newline ('\n').
Applying the output to `grep` solves the annoying ["binary file matches" problem](https://stackoverflow.com/questions/9988379/how-to-grep-a-text-file-which-contains-some-binary-data).

## Aren't there already plenty of solutions?

Yes, but `ascii-filter` appears superior to other solutions when the file is *heavily* corrupted.
Take `./corrupted_lipsum.txt` under this repo as an example.
We will search for "lorem" in the text.
We will use [`ripgrep`](https://github.com/BurntSushi/ripgrep) instead of `grep` for its handy `--count-matches` option.

Solution 1, use `cat -v` to preprocess the file content:

```bash
cat -v corrupted_lipsum.txt | rg -i --count-matches lorem
# output: 21
```

Solution 2, use `grep -a` (or `rg -a`) to force searching:

```bash
cat corrupted_lipsum.txt | rg -a -i --count-matches lorem
# output: 21
```

Solution 3, use `tr` to delete non-printable characters:

```bash
cat corrupted_lipsum.txt | tr -cd '[:print:]' | rg -i --count-matches lorem
# output: 24
```

My solution:

```bash
cat corrupted_lipsum.txt | ascii-filter -a | rg -i --count-matches lorem
# output: 33
```

So, obviously, `ascii-filter` finds the most out of the corrupted text.

## But there's no free lunch ...

`ascii-filter` uses dynamic programming to decide which bytes should be removed, which is of quadratic time complexity.
Therefore, I incrementally process the standard input using sliding window (whose size can be specified by `-b` option), which by default is of size 128.
At the default window size, `ascii-filter` admits a throughput of approximately 467 KB per second.
The smaller the window size, the faster it is, but also the less accurate the filtering algorithm.

## Should I use `ascii-filter`?

You probably don't need it unless the file is really corrupted.
Most of the case, the other solutions mentioned above might suffice.

## How to install?

Make sure you have [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed.
Clone this repo, `cd` into it, and

```bash
cargo install --path .
```
