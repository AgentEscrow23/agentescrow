# COPY AND PASTE THIS PROMPT INTO CLAUDE OR CHATGPT

================================================================
PROMPT FOR SETTING UP YOUR DEV ENVIRONMENT
================================================================

Copy everything below this line and paste it into Claude (claude.ai)
or ChatGPT (chatgpt.com):

---

I'm building a Solana smart contract project called AgentEscrow. I'm completely non-technical — I don't know how to code. I need you to walk me through setting up my development environment step by step.

I'm on [MAC / WINDOWS — pick one].

I need to install:
1. Rust (programming language)
2. Solana CLI (Solana command line tool)
3. Node.js (for testing)
4. Anchor (Solana smart contract framework)

After installing those, I need to:
5. Build my project using "anchor build"
6. Run tests using "anchor test"

I already have the project files downloaded. Here's what I need from you:

- Give me ONE step at a time. Don't give me all the steps at once.
- After each step, ask me "Did that work? What did you see?" before moving to the next step.
- If something fails, help me fix it before moving on.
- Explain everything in very simple terms. I don't know what "PATH" or "terminal" or "compile" means.
- If I need to open a terminal, tell me exactly how to do that on my computer.
- Be patient with me. I'm learning as I go.

Ready? Start with step 1: installing Rust.

---

================================================================
AFTER SETUP IS DONE — PROMPT FOR FIXING THE CODE
================================================================

Once you've installed everything and can run "anchor build," if you get
errors, paste this into Claude/ChatGPT:

---

I'm building a Solana smart contract using the Anchor framework. I just ran "anchor build" and got errors. Here's my code:

[PASTE THE CONTENTS OF lib.rs HERE]

And here's the error I got:

[PASTE THE ERROR MESSAGE HERE]

Can you fix the errors and give me the corrected code? I'm non-technical so please explain what was wrong and what you changed.

---

================================================================
PROMPT FOR UPLOADING TO GITHUB
================================================================

After your code builds and tests pass, paste this into Claude/ChatGPT:

---

I need to upload my Solana project to GitHub. I have a folder called "agentescrow" on my Desktop with these files:
- programs/agentescrow/src/lib.rs (smart contract)
- tests/agentescrow.ts (tests)
- sdk/index.ts (SDK)
- Cargo.toml, Anchor.toml (config files)

I don't have a GitHub account yet. Walk me through:
1. Creating a GitHub account
2. Creating a new repository
3. Uploading my files to it

I'm non-technical. Give me one step at a time and check if it worked before moving on.

---

================================================================
PROMPT FOR DEPLOYING TO SOLANA DEVNET
================================================================

After you've uploaded to GitHub, paste this into Claude/ChatGPT:

---

I have a Solana smart contract that builds and tests successfully on my local machine. Now I want to deploy it to Solana devnet (the free test network) so I can show it working to the Solana Foundation for a grant application.

Walk me through:
1. Getting free SOL on devnet (airdrop)
2. Deploying my Anchor program to devnet
3. Verifying it's deployed

I'm non-technical. One step at a time. Check if each step worked before moving on.

---
