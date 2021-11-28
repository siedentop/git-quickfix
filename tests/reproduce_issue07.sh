#!/usr/bin/env bash

set -ex

export RUST_BACKTRACE=full

REPO="https://github.com/msfjarvis/compose-lobsters"
DIR="compose-lobsters"

if [ -d $DIR ]; then
    rm -rf $DIR
fi

git clone $REPO
cd $DIR

git checkout develop
git reset --soft b6390003b6bb
git reset --hard

# Do some feature development on "feature-x"
git branch feature-x b6390003b6bb
git checkout feature-x
echo "bla bla bla" >> bors.tml
git add bors.tml
git commit -m "Modify bors.tml for fun."

# Modify the README and try to "quickfix it"
echo "This is unofficial" >> README.md
git add README.md
git commit -m "README: clarify unofficial nature"
# We just made a commit on branch 'feature-x' which should be on a different branch

# Here we go:
git quickfix readme-updates --onto develop
cargo run --bin git-qf -- readme-updates --onto develop --force
git log --oneline -3 readme-updates
