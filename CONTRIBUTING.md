# Contributing to Palmeto

Welcome! Palmeto is in desperate need for contributers,
and anyone is welcome!

This document will give you a general idea of how work is done,
as well as what we need done.

Our current focus is the kernel,
as the shell and other user stuff can come much later.

As you may know, this kernel is a hybrid.
It's not exactly a full NT clone,
but it's not UNIX either.

## How to Contribute

Here's some helpful guides on contributing:

- [What To Do?](#what-to-do)
- [How To Contribute?](#how-to-contribute)

## What To Do?

### Bug Fixing

There will obviously be a few bugs in Palmeto,
as all operating systems have.

If you encounter a bug, or just search for one in general,
we will gladly accept a contribution to fix it!

### Working On The Kernel

The kernel is very new and thus not mature,
we need a lot of contributers to focus on the kernel part.

If you have a specific system you want to work on,
then you can send contributions whenever you want!

(e.g... Cache Manager, Memory Management, and more)

### Writing Tests

Tests are very important in kernel development,
and they are also time consuming.

We would love to have contributers who write tests,
It would help us a lot and it isn't as brutal as kernel development.

Feel free to contribute some tests anytime!

### Writing Documentation

This is a big point,
I want to make this project as beginner friendly as possible.

Thus, we would love for you to add documentation.
If you have a specific system, or just in general,
drop it in the docs/ folder and submit it!

## How To Contribute?

Contributions are made super easy by github!

### Pull Requests

You can make a pull request with your code,
and it will be reviewed quickly by a team member!

We also have automated workflows to ensure bad pull requests are avoided,
so make sure to follow the rules!

You can also request a reviewer on your pull request,
which will notify them.

Obviously, it's just me right now.
Meaning your pull request will be reviewed as soon as possible.

Incase you are wondering, all commits must follow a style.
This enforces clean messages, and doesn't cause many issues.

The format is:

```bash
[COMPONENT]: Message
```

by 'COMPONENT' i mean the specific thing you worked on,
for example:

```bash
[RINGBUFFER]: Added is_empty() method
```

please make sure to follow this format,
or else your code cannot be merged.

HOWEVER, you can rename the pull request.
Meaning that you can fix it if you made a mistake.

## Things that need a lot of contributers

Obviously the entire kernel needs contributers,
but there is certain systems that are relied on more than others,
and these ones need maximum attention from contributers.

Here are some systems that I would LOVE for you to contribute to:

- Memory Manager (MOST IMPORTANT)
- Object Manager
- Process Scheduler

## Github Issues

If you don't wish to code and submit a pull request,
we greatly appreciate anyone who will open a issue on github.

Issues are a way of letting developers know about problems,
which will then be fixed by said developers.

Opening an issue, wether it be a bug report,
a new feature request,
or anythingm is greatly appreciated
