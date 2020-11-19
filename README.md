# ZtlNote

ZtlNote is an attempt to implement a zettelkasten organization method software. 

The approach of ZtlNote is to be a fast CLI tool to interact with notes a bit like [Git](https://git-scm.com/). 
The implementation will be a file based store that is organized to maintain strings of notes and references.

## Conception 

### Vocabulary

Organization:
    The Organization is the ensemble of Notes and relations between them.

Note:
    A Note is composed of two distinct parts: a Content and a MetaData.

Note Content:
    Text containing the actual information of a Note. 

Note Meta Data:
    Technical data associated to a Note: date information & links to related notes (parent note, related notes, tags)

Note Address:
    Unique way of locating a Note in the Organization. `[field/]path[:-N]`

Tag:
    Searchable word linked to a Note.

Index:
    File regrouping Notes ID by tag.

Fields:
    A Field is a named field of notes linked by a parent relationship.

Thought Path (abbrev. Path):
    A Path is a named that points to the last Note of a branching in a Field Each time a Note is added to a Path, the Path points to it and the new Note points to its parent forming a path of thoughts.

Store:
    Technical abstraction layer to perform all the I/O needed to persist the Organization state.

Note Identifier:
    Unique technical identifier for a note (UUID V4).

### Location format

A location is an easy way for humans to designate a note at a moment in time. Since this address mode is relative to a branch head and branches are supposed to evolve over time, a note location one day may not designate the same note the day after. Furthermore, a location may be relative to a current field and path. If a unique address stable in time or an absolute address is required then the note UUID shall be used instead. 

`[field/]path[:-N]`

 - field: f specified, the field designate the Tought Field of the Note. If note specified, the current field is used.
 - path: the path followed to reach the Note. Since paths have notes in common, several paths can be used to reach the same note.
 - modifier: the number of ancestor of the branch head's note (default to 0)

 Examples:
    
    field1   A ─ B ─ C - D ←main (current path)
                └ ─ E     ←path1

Assuming the `field1` is the current field, the note B can be reached with the following locations:

 * `main:-2` equivalent of `field1/main:-2`
 * `HEAD:-2` since main is the current path
 * `path1:-1`

Here are other examples of locations:

 * `HEAD` or `main` or `main:-0` or `field1/main` or `field1/main:-0` → note D
 * `path1:-2` → note A
 * `path1:-4` → Nothing
 * `wrongpath` → Nothing
 * `wrong/address/format#` → Error


## Architecture

### front end

The main file uses `StructOpt` to parse the command line arguments. Each command must be a struct testable on its own:

 * info: list current field/path with its date of last note creation/update
 * init: create an Organization and initialize the Store structure on disk.
 * field
    * create: create a field (error if it already exists). This also creates the `main` path in that field (maybe a `--main-path` option may be added in the future to specify the name of that deault branch) `ztln field create field1`
    * list: list thought fields in the Organization (maybe none). `ztln field list`
    * default: set the default field for operations. `ztln field default field1`
 * branch
    * create: create a branch `ztln branch create branch1` (error if it already exists)
    * default: set the given branch as default branch `ztln branch default branch1`
    * list: list the branches in the current field `ztln branch list` or `ztln branch list field1`
 * note:
    * add: create a note from an existing content file `ztln note add filename` or `ztln note add filename branch1` or `ztln note add filename branch1 field1`. Tags and references can be passed as parameters: `ztln note add filename --tags tag1,tag2 --references uuid1,uuid2`
    * show: show a note from a given location `ztln note show "field2/HEAD:-2"` or `ztln note show a4ab9b24-8bff-4b2e-a513-0f489c91f22b` or `ztln note show a4ab9b24`

### library

The `Organization` structure holds the information related to the Organization at least:
 * base_dir
 * default_field
 * store

 It uses the `Store` structure to perform all the I/O in order to persist its state. All methods of the Store API must indicate in which field and path the operations happen. Default field & path are an abstraction of the Organization.

### IO Store

The store is a disk based implementation (like Git):

```
basedir
  +- index ← tag index
  +- notes -+- UUID-1 ← textual content of the notes
  |         +- UUID-2
  |
  +- meta  -+- UUID-1 ← meta data of the notes
  |         +- UUID-2  
  |
  +- fields -+- _CURRENT ← name of the default field
            +- field_1 -+- _HEAD ← name of the default path
            |           +- paths -+- main ← UUID of the last note published in this branch
            |                     +- path1
            |                     +- path2
            |
            +- field_2 -+- _HEAD
                        +- paths -+- main
                                  +- pathN
```

### Tag Store

The tag store manages the `index` file which contains an association of UUID indexed by tags.