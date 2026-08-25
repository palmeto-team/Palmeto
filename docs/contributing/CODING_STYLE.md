# Coding Style

This document will show you the general coding style,
including enforced style rules,
templates,
and more.

Not following this style will generally lead to your pull request being denied,

so make sure to read this.

## File Header

EVERY single source file (rust),
must provide this header at the top.

This makes it easier to understand the reason behind it,
which will greatly help other contributers.

```bash
// SPDX-License-Identifier: GPL-2.0-or-later
//
// Purpose: MESSAGE HERE...
//
```

This header has been reduced in size,
compared to other version of course.

This is due to:

- Removing full copyright message from it
- Removing author section from it
- Removing date section from it

Because github is so powerful,
it can track these things anyways,
making the old-style OUTDATED.

## Functions

EVERY single function (rust),
must provide this header above it.

This makes it easier to understand the reason behind it,
which will greatly help other contributers.

```bash
    ///
    /// PLACE YOUR MESSAGE HERE,
    /// IF IT EXTENDS LIKE IM DOING NOW,
    /// ADD A NEW LINE (///).
    ///
```

HOWEVER.. This comment must change if the function has arguments,
It then becomes:

```bash
    ///
    /// PLACE YOUR MESSAGE HERE,
    /// IF IT EXTENDS LIKE IM DOING NOW,
    /// ADD A NEW LINE (///).
    ///
    /// # Arguments
    /// * arg - Message on what it's for
    ///
```

'arg' being the name of the argument,
seperated by a '-',
followed by a description.

Feel free to format this description onto newlines,
just like the comment.

If your function doesn't have arguments,
only use the top one.

Make sure to use '///'

## Using Snippets

If you're using Visual Studio Code,
then you can use a snippet to store the templates!

This makes it easier to remember them.

If you aren't on Visual Studio Code,
I'm sure there's some way to use Snippets.

But at this time, I am unsure, so be sure to check!

How to use Snippets:

```bash
Combo: 'CTRL' + 'SHIFT' + 'P'

This will open a search bar,
you will type 'Snippets: Configure Snippets'

Then, select 'rust.json',
add the comments to a specific key-combo,
and you're done!
```

Otherwise, you can store them some where on your local machine.
