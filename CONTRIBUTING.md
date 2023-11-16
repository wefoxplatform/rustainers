# Contributing to rustainers

🎈 Thanks for your help improving the project! We are so happy to have
you!

There are opportunities to contribute to `rustainers` at any level. It doesn't
matter if you are just getting started with Rust or are the most weathered
expert, we can use your help.

**No contribution is too small and all contributions are valued.**

This guide will help you get started. **Do not let this guide intimidate you**.
It should be considered a map to help you navigate the process.


## Asking for General Help

If you have reviewed existing documentation and still have questions or are
having problems, you can open an issue asking for help.

In exchange for receiving help, we ask that you contribute back a documentation
PR that helps others avoid the problems that you encountered.

## Requirements

You can use the standard Rust toolchain to work on that project.

To improve your developer experience, you can use [just], and launch `just` to
see available receipts.

Use `just requirement` to install all external requirement.

## Commits

It is a recommended best practice to keep your changes as logically grouped as
possible within individual commits. There is no limit to the number of commits
any single Pull Request may have, and many contributors find it easier to review
changes that are split across multiple commits.

Note that multiple commits often get squashed when they are landed.

Commit messages should follow [Conventional commits].


## Release process

For the moment it's a manual process:

1. go to the `main` branch, ensure that's the branch is up-to-date: `git checkout main; git pull`
1. run checks: `just check`
1. update the top-level `Cargo.toml` version to "x.y.z"
1. update the `CHANGELOG.md` (you can use [git cliff] to help the generation)
1. commit changes `git commit -am 'release: version x.y.z'`
1. tag the project with `git tag 'x.y.z' -m 'release: version x.y.z'`
1. push to github `git push`
1. publish to [crate.io] (you need to be logged)
1. (announce the new version & celebrate)

[just]: https://just.systems/
[Conventional commits]: https://www.conventionalcommits.org/
[crate.io]: https://crates.io/
[git cliff]: https://github.com/orhun/git-cliff
