# Find the commit that did it

`git bisect` halves the range at every step: you name a revision where the behaviour is and one where it is not, and after roughly log₂(n) answers you have the commit that introduced it.

## The half git cannot know

`git bisect` moves the **code** and nothing else.

Three months ago this project declared PHP 8.3 and locked `redis` at 7.0. The container on this machine today is 8.4 and 7.2. So every step through that range runs **old code against a new environment** — and the commit the search finally accuses may be innocent, because the behaviour changed with the runtime rather than with the diff.

Nothing else in this category does anything about that, because nothing else knows what environment a commit wanted. This does: `stackvo.json` has always travelled with the repository, and since `stackvo.lock` the service versions do too. Both are read **at the revision under test**, without touching your working tree, and what differs is listed under the commit.

"This machine matches what this commit expected" is a result rather than an empty space. It means the environment is not in your bisect, so whatever the search accuses is the code.

## Nothing is changed for you

There is no button that brings the environment along, and that is deliberate. Matching an old service version means replacing a container whose volume holds your data — and a ten-step bisect would do it twenty times. Destroying a database to answer a question about a diff is not a trade this app makes on your behalf.

The listing is a sentence, and acting on it is your decision. The Market page is where to make it, and it asks first.

## Three buttons, not two

| Button | When |
| --- | --- |
| **Broken here** | The behaviour you are hunting is present. |
| **Works here** | It is not. |
| **Cannot test this one** | This commit does not build, or the feature does not exist yet. |

The third one is git's own `skip` and it matters more than it looks. Without it, a commit that will not build gets marked *works here* to move past it — which poisons the search in a way nothing downstream can detect.

## What is refused, and why

- **Uncommitted changes.** A bisect walks your checkout through other people's commits. Commit or stash first. This refuses before anything moves, by name, rather than letting a git error reach you as a sentence written for a terminal.
- **A revision that is not one.** What you type reaches `git` as an argument, and a value beginning with `-` is read by git as an *option* — of which git has several that name a program to run. Only git's own revision alphabet is accepted: `main`, `v1.2.3`, `HEAD~5`, `origin/main`, `abc1234`.

## Worth knowing

- **Stop and put my checkout back** runs `git bisect reset`, which returns you to the branch you started from. It stays available after the answer is found, because that screen is a detached HEAD like every other step.
- Starting a bisect and marking a step are both recorded in the audit trail. "My files are not what they were" is the loudest question a developer can have about their own machine, and the answer has to be somewhere.
- The step estimate is git's, read back rather than recomputed here — it depends on the shape of the history, on skips and on merge parents.
- A commit from before `stackvo.json` or `stackvo.lock` existed lists no differences. The bisect still works over that range; it simply has no environment half there, which is where every other tool always is.
