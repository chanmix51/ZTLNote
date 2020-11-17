# ZtlNote

ZtlNote is an attempt to implement a zettelkasten organisation method software. 

The approach of ZtlNote is to be a fast CLI tool to interact with notes a bit like [Git](https://git-scm.com/). 
The implementation will be a file based store that is organized to maintain strings of notes and references.

# Conception 

## Vocabulary

Organisation:
    The Organisation is the ensemble of Notes and relations between them.

Note:
    A Note is composed of two distinct parts: a Content and a MetaData.

Note Content:
    Text containing the actual information of a Note. 

Note Meta Data:
    Technical data associated to a Note: date information & links to related notes (parent note, related notes, tags)

Note Address:
    Unique way of locating a Note in the Organisation. `[tree/]path[:-N]`

Tag:
    Searchable word linked to a Note.

Index:
    File regrouping Notes ID by tag.

Thought Tree:
    A Thought Tree is a named tree of notes linked by a parent relationship.

Thought Path:
    A path is a named unique branch of Notes in a Thought Tree.

Store:
    Technical abstraction layer to perform all the I/O needed to persist the Organisation state.

Note Identifier:
    Unique technical identifier for a note (UUID V4).

### Location format

A location is an easy way for humans to designate a note at a moment in time. Since this address mode is relative to a branch head and branches are supposed to evolve over time, a note location one day may not designate the same note the day after. Furthermore, a location may be relative to a current tree and path. If a unique address stable in time or an absolute address is required then the note UUID shall be used instead. 

`[tree/]path[:-N]`

 - tree: f specified, the tree designate the Tought Tree of the Note. If note specified, the current tree is used.
 - path: the path followed to reach the Note. Since paths have notes in common, several paths can be used to reach the same note.
 - modifier: the number of ancestor of the branch head's note (default to 0)

 Examples:
    
    tree1   A ─ B ─ C - D ←main (current path)
                └ ─ E     ←path1

Assuming the `tree1` is the current tree, the note B can be reached with the following locations:

 * `main:-2` equivalent of `tree1/main:-2`
 * `HEAD:-2` since main is the current path
 * `path1:-1`

Here are other examples of locations:

 * `HEAD` or `main` or `main:-0` or `tree1/main` or `tree1/main:-0` → note D
 * `path1:-2` → note A
 * `path1:-4` → Nothing
 * `wrongpath` → Nothing
 * `wrong/address/format#` → Error


## Architecture

The main file uses `StructOpt` to parse the command line arguments. Each command must be a struct testable on its own:

 * info: list current tree/path with its date of last note creation/update
 * organisation (org)
    * init: create an organisation and initialize the Store structure on disk.
 * tree
    * create: create a thought tree (error if it already exists). This also creates the `main` branch in that tree (maybe a `--main-branch` option may be added in the future to specify the name of that deault branch) `ztln tree create tree1`
    * list: list thought trees in the organisation (maybe none, error if no org could be found). `ztln tree list`
    * default: set the default tree for operations. `ztln tree default tree1`
 * branch
    * create: create a branch `ztln branch create branch1` (error if it already exists)
    * default: set the given branch as default branch `ztln branch default branch1`
    * list: list the branches in the current tree `ztln branch list` or `ztln branch list tree1`
 * note:
    * add: create a note from an existing content file `ztln note add filename` or `ztln note add filename branch1` or `ztln note add filename branch1 tree1`. Tags and references can be passed as parameters: `ztln note add filename --tags tag1,tag2 --references uuid1,uuid2`
    * show: show a note from a given location `ztln note show "tree2/HEAD:-2"`

The `Store` struct is the key object to manage an Organisation.