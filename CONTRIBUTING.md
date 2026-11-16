# Contributing

**Anyone can contribute to BIMCraft.** You don't need permission and you don't need to be an expert. You make your change
in your own copy of the project and send it as a **pull request** (PR). The maintainers review it, and once it's approved it
goes into BIMCraft for everyone.

BIMCraft is written with AI agents. Most of its code was written by people working with a coding agent. You can do the same,
even if you have never written Rust. This guide shows you how, step by step. It is also on the
[wiki](https://github.com/IPedrax/BIMCraft/wiki/Contributing), with the user guide for every tool.

**In this guide:** [Ways to help](#ways-to-help) · [What you need](#what-you-need) ·
[Your first contribution, step by step](#your-first-contribution-step-by-step) · [Rules every change follows](#rules-every-change-follows) ·
[A good pull request](#a-good-pull-request) · [Wiki pages](#wiki-pages) · [After you send it](#after-you-send-it) · [For agents](#for-agents)

---

## We get it

**Everyone has a pain point or bright idea they want to see in the software as soon as possible. Having agents do it for you quickly is just the cherry on the cake. However, please understand that not everything can be implemented and not as soon as possible.**

1. **There is a sequence of operations. For example, you cannot have Civil3D landscape import before we figure out coordinates and if they are working. Start to work on the actual foundations first to reach your final goal.**
2. **Your idea may collide with an existing one. This is a collaborative project and we are sitting here with an open mind. Everything can be argued over or improved upon. You might have a great idea but others can still give you great insights on how to make it better. Post about it, ask for review or comments.**
3. **This is an international project. Submitting tiny little tweaks to fit your local code is not welcome. Instead think in a system: "How can we make the software work with any national code?" These kinds of contributions are very welcome.**
4. **Be kind, be civil and be professional. If your idea is not accepted, the software still supports plugins. Just create one for yourself to suit your needs.**

## Ways to help

You don't have to write code to help.

| You can | How |
|---|---|
| **Report a bug** | Open an [issue](https://github.com/IPedrax/BIMCraft/issues/new/choose) with the Bug report form. Say what you did, what you expected and what happened. Steps that start from the sample house (`bimcraft --sample`) are the easiest to follow. |
| **Suggest an idea** | Start a thread in [Discussions > Ideas](https://github.com/IPedrax/BIMCraft/discussions/categories/ideas). |
| **Improve this wiki** | Fix a mistake, explain something better, add a picture. See [Wiki Plan](https://github.com/IPedrax/BIMCraft/wiki/Wiki-Plan) for how pages are written. |
| **Translate** | BIMCraft speaks many languages. The words live in `translations/` as `.po` files, which any translation tool opens. |
| **Fix a bug or add a feature** | Make the change with your agent and open a pull request. The rest of this page is about this. |

---

## What you need

1. **A GitHub account.** Free, at [github.com/signup](https://github.com/signup).
2. **Git**, to copy the project and send your changes. Get it from [git-scm.com](https://git-scm.com/downloads).
3. **Rust**, to build BIMCraft. Install it with [rustup](https://rustup.rs). BIMCraft needs Rust 1.95 or later for the desktop app.
4. **The GitHub CLI** (`gh`), optional but handy: it lets your agent open the pull request for you. From [cli.github.com](https://cli.github.com).
5. **An AI coding agent** that can read and change files and run commands in a folder on your computer, such as
   [Claude Code](https://claude.com/claude-code).
6. **A strong enough model.** Use at least one of these, or a bigger or equivalent model:

   | Model | Setting |
   |---|---|
   | **Claude Opus 5.5** | **high** effort |
   | **Sol 6.1** | **high** effort |

   **Why this matters:** BIMCraft is a large project (more than 200,000 lines of Rust) with strict rules: it must never
   crash, every change needs a test, and nothing may be copied from Autodesk. Smaller or faster models tend to miss these
   rules, break things elsewhere, or write tests that don't really test anything. A pull request made with a weaker model
   usually costs the reviewers more time than it saves. If you are not sure your model is strong enough, it probably isn't.

   In Claude Code, pick the model with `/model` and set the effort to high. Other agents have a similar setting.

---

## Your first contribution, step by step

### Step 1: Fork the project

A **fork** is your own copy of BIMCraft on GitHub. You can change it freely.

1. Go to [github.com/IPedrax/BIMCraft](https://github.com/IPedrax/BIMCraft).
2. Click **Fork** at the top right, then **Create fork**.

You now have `github.com/<your-name>/BIMCraft`.

### Step 2: Copy it to your computer

Open a terminal and type (put your GitHub name in place of `<your-name>`):

```sh
git clone https://github.com/<your-name>/BIMCraft.git
cd BIMCraft
git remote add upstream https://github.com/IPedrax/BIMCraft.git
```

The last line remembers where the original project is, so you can get its newest changes later.

### Step 3: Build it and see it run

```sh
cargo run --release -p bimcraft -- --sample
```

The first build takes a while (often 10 to 20 minutes). When it finishes, the desktop app opens on the sample house. If it
does, your computer is ready. If it doesn't, ask your agent to help: paste it the error.

### Step 4: Choose something to work on

Start small. A small change that is done well teaches you the project and gets merged quickly.

- An [issue labelled bug](https://github.com/IPedrax/BIMCraft/issues?q=is%3Aissue+is%3Aopen+label%3Abug) you can reproduce.
- Something you found yourself while using BIMCraft.
- An item from [ROADMAP.md](ROADMAP.md).

**Before you start on anything bigger than a small fix, say so on the issue** (or open one). Someone may already be
working on it, and the maintainers can tell you how they'd like it done. That saves you work.

### Step 5: Make a branch

A **branch** keeps your change apart from everything else. Name it after what you're doing:

```sh
git checkout main
git pull upstream main
git checkout -b fix/door-in-curved-wall
```

### Step 6: Start your agent in the project folder

Start your agent in the `BIMCraft` folder (for Claude Code, type `claude` there). Check the model and effort ([What you
need](#what-you-need), point 6).

BIMCraft has instruction files written for agents: `CLAUDE.md` at the top, and more `CLAUDE.md` files and `docs/areas/*.md`
pages for each part of the code. Claude Code reads `CLAUDE.md` by itself. If your agent doesn't, tell it to read it first.

### Step 7: Tell the agent what to do

Describe the change in plain words. Be specific: what happens now, what should happen, and how to see it. A good first
message looks like this:

```text
Read CLAUDE.md first and follow its rules.

The problem: when I place a door in a curved wall, the door is placed at the wrong spot.
Steps: open the sample house, draw a curved wall with Wall > Start-End-Radius Arc,
then use Door and click the middle of the curved wall. The door appears near the end instead.

Please:
1. Find where this happens and explain the cause to me in simple words.
2. Write a test that fails because of this bug, and run it to show it fails.
3. Fix it, and show the test passing.
4. Look at the result as docs/areas/looking-at-results.md says (render the plan and check it).
5. Run cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace.
6. Tell me what you changed and why.
```

Let the agent work. When it asks you a question, answer it. When it explains something you don't understand, ask it to
explain again more simply. **You are responsible for the change**, so make sure you understand what it does.

### Step 8: Check the result yourself

Tests are not enough. Look at it with your own eyes:

1. Run the app again (`cargo run --release -p bimcraft -- --sample`) and try what you changed, the way a user would.
2. Try one or two things near it, to be sure nothing else broke.
3. Ask the agent: *"What could this change break? Check those places."*

### Step 9: Run the three checks

Every pull request must pass these. Your agent should already have run them, but run them once more yourself:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

All three must finish without errors. If one fails, give the output to your agent and ask it to fix the cause (not to
hide the warning).

### Step 10: Save your change (commit)

```sh
git status                 # see which files changed
git add <the files you changed>
git commit -m "Doors in curved walls are placed where you click"
```

Write the message as what is now true, in one plain line. Your agent can write it for you. Add only the files that belong
to your change.

### Step 11: Send it to your fork

```sh
git push -u origin fix/door-in-curved-wall
```

### Step 12: Open the pull request

1. Open your fork on GitHub. A yellow bar says your branch has recent pushes: click **Compare & pull request**.
2. Check that it goes **from** your branch **into** `IPedrax/BIMCraft` `main`.
3. Write a title and a description (see [A good pull request](#a-good-pull-request)).
4. Click **Create pull request**.

With the GitHub CLI, your agent can do this for you: ask it to *"open a pull request to IPedrax/BIMCraft with a clear
description"*.

### Step 13: Wait for the checks and the review

- **CI** (automatic checks) builds and tests your change on Linux, Windows and macOS. For your first pull request a
  maintainer has to approve the run first, so it may wait a little.
- A maintainer reads your change and may ask for changes. Give their comments to your agent, make the changes, commit and
  push again to the same branch. The pull request updates by itself.
- When everything is approved, a maintainer merges it. Congratulations, you are a BIMCraft contributor.

---

## Rules every change follows

These come from `CLAUDE.md`. Your agent knows them, but you should know them too, because the review checks them.

| Rule | In plain words |
|---|---|
| **Never crash** | BIMCraft must never crash, whatever a user types or whatever file they open. Code may not use shortcuts that crash when something is missing (`unwrap`, `expect`, `panic!` and the like). |
| **Test first** | Write a test that fails, watch it fail, then fix the code. Every new behaviour has a test. |
| **Everything is a command** | Anything a user can do is a command, so the desktop, scripts and agents all do it the same way. New behaviour is a new command or a new parameter. |
| **Validate in one place** | The model checks itself after every command and undoes a change that breaks a rule. New rules go there, with a message that says how to fix the problem. |
| **The core has no interface** | The engine (`crates/core`) knows nothing about windows or buttons. The desktop only shows the model and runs commands. |
| **Clean-room** | **Never copy anything from Autodesk**: no families (`.rfa`), templates, icons, help text, or content from `.rvt` files. Don't paste Revit's help into a prompt. Formats come from public specifications. |
| **Keep the docs true** | If you change how something works, change its notes in `docs/areas/` and, if it's for users, its page in the [wiki](https://github.com/IPedrax/BIMCraft/wiki) (see [Wiki pages](#wiki-pages)). |

Don't attach Autodesk files to issues or pull requests either.

## A good pull request

- **One thing per pull request.** A bug fix, or one feature. Small pull requests are reviewed quickly.
- **A clear title**, as what is now true: *"Doors in curved walls are placed where you click"*.
- **A description** that says:
  - **What** changed and **why** (link the issue: `Fixes #123`).
  - **How you tested it**: the tests added, and what you tried in the app.
  - **A picture** for anything you can see: a screenshot of the window or a rendered plan, before and after.
  - **Which agent and model** you used (for example *Claude Code, Opus 5.5, high effort*).
- **Passing checks**: the three commands above.
- **Its wiki page**, if users will see the change (next section).

The pull request template lists these, so you can tick them off.

## Wiki pages

The [wiki](https://github.com/IPedrax/BIMCraft/wiki) is the user guide: one page for each tool, in plain words for people,
with exact commands for agents. Users read the wiki, not the code, so **a feature users can see is not finished until its
page says how to use it.**

- **A new tool or command:** add it to the page of its area (or write a new page, if no page fits) and to the sidebar.
- **A change to how something works:** update the page that explains it, when the change goes in.
- **A bug fix, a refactor or a test:** usually no wiki change.

How a page is written (sections, plain English, pictures made with `bimcraft-cli`, examples that have been run) is in
[Wiki Plan](https://github.com/IPedrax/BIMCraft/wiki/Wiki-Plan). The wiki is its own git repository,
`https://github.com/IPedrax/BIMCraft.wiki.git`, so your agent can write the page too: ask it to *"write the wiki page for
this change, following Wiki Plan"*. If you can't edit the wiki yourself, put the page's text in your pull request
description under **Wiki page**, and a maintainer will add it.

## After you send it

- Be patient: the maintainers review in their own time.
- Keep your branch up to date if asked: `git pull upstream main`, then push again.
- If the review asks for a bigger change than you expected, that's normal. Ask questions in the pull request.

## License

BIMCraft is licensed under MIT OR Apache-2.0 (the Revit reader, `crates/rvt`, is Apache-2.0 only). By opening a pull
request you agree that your contribution is licensed the same way.

---

## For agents

You are helping a contributor send a change to BIMCraft. Follow this, in order:

1. **Read the rules.** `CLAUDE.md` at the repository root, then the `CLAUDE.md` of the crate you touch
   (`crates/core/CLAUDE.md`, `crates/ui-egui/CLAUDE.md`, …) and the `docs/areas/<area>.md` the module map names for it.
2. **Work on a branch** of the contributor's fork, named for the change (`fix/…`, `feat/…`), never on `main`.
3. **Test first.** Write the failing test, run it and show it failing, then write the code. Engine behaviour is tested
   through commands (`Session::execute`) in `crates/core/src/tests.rs` or the area's test file; desktop behaviour with
   egui_kittest in `crates/ui-egui/src/tests.rs`; drawings and IFC through their output (PNG pixels, STEP references).
4. **New behaviour is a command or a parameter** in `ALL` (`crates/core/src/commands.rs`), with a params doc written for
   agents. Hostile input never panics: check numbers are finite and in range, cap sizes.
5. **Validation goes in `Model::check`**, with a message that says what to fix.
6. **Look at the result** as `docs/areas/looking-at-results.md` says for that kind of output, and show it to the contributor.
7. **Run the checks** before declaring it done, and report their real output:

   ```sh
   cargo fmt --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```

8. **Update the docs**: the area's write-up in `docs/areas/`, the module map in `CLAUDE.md` for a new module (it must stay
   under 3,000 words; the `agent_docs` test checks), `README.md` if users see it, and the wiki page for the tool ([Wiki pages](#wiki-pages)).
9. **Clean-room.** Never fetch, quote or paraphrase Autodesk documentation, families, templates or `.rvt` contents into the
   code. Revit file reading lives only in `crates/rvt`, under `docs/rvt-provenance.md`.
10. **Explain** the change to the contributor in plain words, so they can answer the review.
11. **Commit** only the files of the change, staged by name, with a message stating what is now true. Open the pull request
    against `IPedrax/BIMCraft` `main` with: what and why, the issue it fixes, how it was tested, a picture for anything
    visible, the wiki page written or updated, and the agent and model used. Fill in the pull request template.

**Model floor:** this work needs Claude Opus 5.5 at high effort, Sol 6.1 at high effort, or a bigger or equivalent model.
If you are running on a smaller model, say so to the contributor before starting.
