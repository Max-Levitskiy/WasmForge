# Base rules

- Track all your work in MD files


# English as a source code

We use human language as a source code. We describe logic in the md files before start the implementation, then try to understand complexity of the current expected logic, and split it to submodules, if needed. Do it until we have description that fits DRY and SOLID principles. 

Update sources proactivly. If you see something is outdated - update it. If you touch the module, validate the documentation and check, if it still up-to-date.

## Files format to track

In docs/ai/source we store all our AI source code. 

In each directory, we can have the following files:
- brief.md - short description of current level abstraction. It has relations description, can have diagrams, list of submodules (subfolders). If it's hard to have it less than than 1500 symbols, it's good reason to refactor the module and split.
- kanban.md - contains planned, in progress and on review tasks.
- worklog.md - contains archive of completed tasks and important notes we . When finish task from kanban, append it here with "---" section spit. The file is "append-only". We don't remove anything from there.
- full-description.md - long description of the module. Contains important decisions we did while we  

## Tree structure 
We try to keep each level module small and clean. If it start to begin to be big, we refactor it and split to smaller pieces.