# ZtlNote

ZtlNote is an attempt to implement a zettelkasten organization method software. 

The approach of ZtlNote is to be a fast CLI tool to interact with notes a bit like [Git](https://git-scm.com/). 
The implementation will be a file based store that is organized to maintain strings of notes and references.

## Usage

### Introduction

In his book about the Zettelkasten organization (*How to Take Smart Notes*), Ahrens Sönke describes 3 steps:

- take flying notes about what you read/hear/see 
- rework these notes in the Zettelkasten combining them with the existing ones and trash flying notes
- when writing about a topic, extract paths of thoughts from the Zettelkasten as raw material.

The forces of the Zettelkasten lie in its organization in paths, references between notes and a taggin system. ZtlNote combine these three things.

I have written ZtlNote with my understanding of the book in an effort of testing this organization while having a coding experience with Rust.

### Command line

    ztln command [arguments] [options]

 * info: list current topic/path with its date of last note creation/update
 * init: create an Organization and initialize the structure on disk.
 * topic
    * create: create a topic. This also creates the `main` path in that topic (maybe a `--main-path` option may be added in the future to specify the name of the topic's default path) `ztln topic create TOPIC`
    * list: list topics in the Organization (maybe none). `ztln topic list`
    * default: set the default topic for operations. `ztln topic default TOPIC`
 * path
    * branch: branch a path from a specified note `ztln path create PATH [LOCATION]`
    * list: list the existings paths in the current topic `ztln path list` or `ztln path list TOPIC`
    * default: set the given path as default path `ztln path default PATH`
 * note:
    * add: create a note from an existing content file `ztln note add [FILENAME] [-p PATH] [-t TOPIC]`
    * show: show a note from a given location `ztln note show LOCATION`.
    * reference: create a reference from one note to another `ztln note reference LOCATION LOCATION`
 * tag
    * add: tag a note with the given keyword `ztln tag add KEYWORD [LOCATION]`
    * search: search all notes tagged with the given keyword `ztln tag search KEYWORD`
    * list: list all the keywords stored in the index `ztln tag list`

### Location format

A location is an easy way for humans to designate a note at a moment in time. Since this address mode is relative to a head and paths are supposed to evolve over time, a note location one day may not designate the same note the day after. Furthermore, a location may be relative to a current topic and path. If a unique address stable in time or an absolute address is required then the note UUID shall be used instead. 

`[topic/]path[:-N]`

 - topic: f specified, the topic designate the Tought Topic of the Note. If note specified, the current topic is used.
 - path: the path followed to reach the Note. Since paths have notes in common, several paths can be used to reach the same note.
 - modifier: the number of ancestor of the path head's note (default to 0)

 Examples:
    
    topic1   A ─ B ─ C - D ←main (current path)
                 └ ─ E     ←path1

Assuming the `topic1` is the current topic, the note B can be reached with the following locations:

 * `main:-2` equivalent of `topic1/main:-2`
 * `HEAD:-2` since main is the current path
 * `path1:-1`

Here are other examples of locations:

 * `HEAD` or `main` or `main:-0` or `topic1/main` or `topic1/main:-0` → note D
 * `path1:-2` → note A
 * `path1:-4` → Nothing
 * `wrongpath` → Nothing
 * `wrong/address/format#` → Error


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
    Unique way of locating a Note in the Organization. `[topic/]path[:-N]`

Tag:
    Searchable word linked to a Note.

Index:
    File regrouping Notes ID by tag.

Topics:
    A Topic is a named topic of notes linked by a parent relationship.

Thought Path (abbrev. Path):
    A Path is a name that points to the last Note of a branching in a Topic. Each time a Note is added to a Path, the Path points to it and the new Note points to its parent forming a path of thoughts.

Store:
    Technical abstraction layer to perform all the I/O needed to persist the Organization state.

Note Identifier:
    Unique technical identifier for a note (UUID V4).

## Architecture

### front end

The main file uses `StructOpt` to parse the command line arguments. Each command must be a struct testable on its own:

### library

The `Organization` structure holds the information related to the Organization at least:
 * base_dir
 * default_topic
 * store

 It uses the `Store` structure to perform all the I/O in order to persist its state. All methods of the Store API must indicate in which topic and path the operations happen. Default topic & path are an abstraction of the Organization.

### IO Store

The store is a disk based implementation:

```
basedir
  +- index ← tag index
  +- _CURRENT ← name of the default topic when exist
  +- notes -+- UUID-1 ← textual content of the notes
  |         +- UUID-2
  |
  +- meta  -+- UUID-1 ← meta data of the notes
  |         +- UUID-2  
  |
  +- topics -+- topic_1 -+- _HEAD ← name of the default path when exist
             |           +- description ← long description of the topic when exist
             |           +- paths -+- main ← UUID of the last note published in this path
             |                     +- path1
             |                     +- path2
             |
             +- topic_2 -+- _HEAD
                         +- paths -+- main
                                   +- pathN
```

### Tag Store

The tag store manages the `index` file which contains an association of UUID indexed by tags.
