# Blockito (little blocky)

This project is a proof of concept for a rag system
on top of our customer service training data.

Currently, `cat-facts.txt` is our fake training data.

## Getting Started

Install [`rust` and `cargo`](https://rust-lang.org/learn/get-started/).

We are currently fairly tightly coupled with `frontend-csp` in the frontend
monorepo, (though not completely, just for this POC).

We recommend cloning that project and following the
installation instructions to get that up and running.

The main branch we've been using for
[that project](https://github.com/FigureTechnologies/frontend) is
`wlabranche/no-ticket/robot`. It's currently a bit behind `main`,
but nothing much to worry about there.

To run the project, you have two choices:

`cargo run` from inside the root directory.

This should everything for you, but this will not
automatically restart the server on change.

To have that functionality, you will want to install
[`bacon`](https://dystroy.org/bacon/), with `cargo install --locked bacon`.

Then from the root directory:

```
bacon run
```

You will probably see a couple weird errors,
this is because we need to get the LLM server up and running.

In a different terminal window or pane, we will need to install and run our LLM server.

There are a lot of options here, you can use whatever you'd like,
but I'd recommend (and what I will be showing here) is
[`llama.cpp`](https://github.com/ggml-org/llama.cpp) and [`llama-swap`](https://github.com/mostlygeek/llama-swap).

These are light weird and lot of other tooling is just built on top of these.

I'm also assuming you're using a mac, so install with `brew`:

```
brew install llama.cpp
```

This will install the `llama.cpp` suite,
that includes `llama-server` and `llama-cli`,
we mostly care about `llama-server` here.

Then install `llama-swap` with:

```
brew tap mostlygeek/llama-swap
brew install llama-swap
```

`llama-swap`, is a basic server that theoretically wraps `llama.cpp`,
but can be used with other things as well, but don't stress to much there.

It's set up is very straightforward and consists of just a config file.

I store mine at `~/.config/llama-swap/config.yaml`.

Once all that is installed,
you can just copy my current config file and it should work fine.

```
models:
  "blockito-language":
    cmd: |
      llama-server
      -hf unsloth/Qwen3-0.6B-GGUF
      --port ${PORT}
  "blockito-embedding":
    cmd: |
      llama-server
      -hf CompendiumLabs/bge-base-en-v1.5-gguf
      --embedding
      --port ${PORT}
groups:
  "blockito":
    swap: false
    exclusive: true
    members:
      - "blockito-language"
      - "blockito-embedding"
```

If you're curious about what is happening here,
the `models` group just aliases our server commnads to start when needed.

Here we're using two models, one aliased as `blockito-language` and
the other as `blockito-embedding` and set the `llama-server` command
to run with the appropriate model.

`-hf` defines the model and where to get it, here it's from Hugging Face
and will automatically download it the first time it's run.
You can find other options on the `llama.cpp` GitHub page.

`--port` is fairly self explanatory, it tells the server what port to run on and
`llama-swap` will automatically set the `${PORT}`
environment variable when starting the server.

`--embedding` tells `llama-server` to run in embedding mode,
which is what we want for the embedding model (I'm still learning more about this).

We then have the `groups` section where we can
define groups of models to by combined on the same server.

First, we define a group called `blockito`,
which is what our `rust` server will connect to.

`swap` basically means if the two servers will run
independently or one at a time, here we set it to `false` so both can run.
I need to learn a bit more about this.

`exclusive` means defines if we have more than one group, if they can interact,
we don't need that, `exclusive: true` is the default,
but I'm learning so I like it explicit.

Then, `members` is just a list of the models we defined earlier
that will be part of this group and matches the models we defined above.

Once this is all setup, start the llama server with:

```
llama-swap --config ~/.config/llama-swap/config.yaml --listen localhost:8765
```

This will start the server on port `8765` on your local machine.
I have this aliased in my shell to `blockito-server`.

Eventually, we will wan that port
(and the port in the rust server to be configurable)
and we'll also want to do a better job of defining
and managing which models we interact with.

`./src/consts.rs` is probably doing a bit too much right now.

Once we have the llama server running, we can return to our rust server.

If it's still running, just stop it and restart it with `bacon run` again.

It will automatically read our `cat-facts.txt` and build the vector database,
if something is funky, shut it down and delete the generated `sqlite` files
and start again.

With all that running, get frontend csp running and navigate to `http://localhost:3000/csp`.

The chat window will now be set up to talk to blockito about cats.

So, in short, once everything is set up:

`llama-swap --config ~/.config/llama-swap/config.yaml --listen localhost:8765`
in it's own terminal pane or window.

`bacon run` from rust project.

`one start -w csp` from the frontend monorepo.

Then navigate to `http://localhost:3000/csp` in your browser.
