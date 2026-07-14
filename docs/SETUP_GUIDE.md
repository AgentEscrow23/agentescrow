# AgentEscrow — Development Environment Setup Guide
# FOR NON-TECHNICAL PEOPLE — NO CODING EXPERIENCE NEEDED

================================================================
STEP 0: WHAT YOU NEED BEFORE STARTING
================================================================

You need a computer (Mac or Windows) and internet. That's it.

If you're on Windows, you'll need to install WSL (Windows Subsystem for Linux) first.
If you're on Mac, you're ready to go.

================================================================
STEP 1: INSTALL RUST (The programming language Solana uses)
================================================================

MAC:
  1. Open Terminal (press Command + Space, type "Terminal", hit Enter)
  2. Copy and paste this exact command and press Enter:

     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

  3. It will ask "1) Proceed with installation (default)"
     Type 1 and press Enter
  4. Wait for it to finish (2-3 minutes)
  5. When it's done, type this and press Enter:

     source "$HOME/.cargo/env"

  6. Verify it worked by typing:

     rustc --version

     You should see something like "rustc 1.78.0" — that means it worked.

WINDOWS:
  1. First install WSL by opening PowerShell as Administrator and running:

     wsl --install

  2. Restart your computer
  3. Open Ubuntu (it'll be in your Start menu)
  4. From now on, follow the MAC instructions above inside the Ubuntu terminal

================================================================
STEP 2: INSTALL SOLANA CLI (The Solana command line tool)
================================================================

MAC (in Terminal):
  1. Copy and paste this command and press Enter:

     sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

  2. Wait for it to finish (3-5 minutes)
  3. When it's done, it will tell you to add Solana to your PATH.
     It will show you a command that looks like:

     export PATH="/Users/yourname/.local/share/solana/install/active_release/bin:$PATH"

  4. Copy that exact command and paste it in Terminal, press Enter
  5. Verify it worked by typing:

     solana --version

     You should see something like "solana-cli 1.18.0" — that means it worked.

WINDOWS:
  Follow the same instructions but inside your Ubuntu terminal (from Step 1)

================================================================
STEP 3: INSTALL NODE.JS (Needed for Anchor and testing)
================================================================

MAC (in Terminal):
  1. Copy and paste this command and press Enter:

     curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash

  2. Close your Terminal and open a new one
  3. Type this and press Enter:

     nvm install 20

  4. Verify it worked by typing:

     node --version

     You should see something like "v20.11.0" — that means it worked.

WINDOWS (in Ubuntu terminal):
  Same instructions as Mac

================================================================
STEP 4: INSTALL ANCHOR (The framework for Solana smart contracts)
================================================================

MAC (in Terminal):
  1. Copy and paste this command and press Enter:

     cargo install --git https://github.com/coral-xyz/anchor avm --locked --force

  2. Wait for this to finish (this takes 10-15 minutes, be patient)
  3. When it's done, type:

     avm install 0.30.1

  4. Then type:

     avm use 0.30.1

  5. Verify it worked by typing:

     anchor --version

     You should see "anchor-cli 0.30.1" — that means it worked.

WINDOWS (in Ubuntu terminal):
  Same instructions as Mac

================================================================
STEP 5: SET UP SOLANA FOR LOCAL TESTING
================================================================

MAC (in Terminal):
  1. Type this and press Enter:

     solana config set --url localhost

  2. Type this and press Enter (creates a test wallet):

     solana-keygen new --no-bip39-passphrase

  3. It will show you a "pubkey" — this is your test wallet address.
     You don't need to save this, it's just for local testing.

================================================================
STEP 6: DOWNLOAD THE AGENTESCROW PROJECT
================================================================

  1. Download the project zip file (you already have this)
  2. Unzip it
  3. Move the "agentescrow" folder to your Desktop
  4. In Terminal, navigate to the project by typing:

     cd ~/Desktop/agentescrow

  5. You should now be inside the project folder.
     Type "ls" and press Enter — you should see:
     Anchor.toml  Cargo.toml  README.md  docs  programs  sdk  tests

================================================================
STEP 7: BUILD THE PROJECT
================================================================

  1. In Terminal (inside the agentescrow folder), type:

     anchor build

  2. This will take 5-10 minutes the first time (it's downloading dependencies)
  3. If you see errors, DON'T PANIC. This is normal for the first build.
     Copy the error message and paste it into Claude or ChatGPT and ask:
     "I got this error when running anchor build, how do I fix it?"

  4. If it succeeds, you'll see "Finished" at the end.

================================================================
STEP 8: TEST THE PROJECT
================================================================

  1. First, start the local Solana network. In a NEW Terminal window, type:

     solana-test-validator

  2. Leave that window open (it's running your local test network)
  3. Go back to your first Terminal window and type:

     anchor test

  4. This runs the test suite. If everything works, you'll see:
     "AgentEscrow
       ✓ Creates an escrow
       ✓ Funds the escrow
       ✓ Provider starts work
       ✓ Provider submits work
       ✓ Payer confirms completion and funds release to provider

     5 passing"

  5. If you see errors, copy them and ask Claude/ChatGPT for help fixing them.

================================================================
STEP 9: UPDATE THE PROGRAM ID
================================================================

After your first successful build, Anchor generates a real program ID.
You need to update it in the code:

  1. Type this to find your program ID:

     solana address -k target/deploy/agentescrow-keypair.json

  2. Copy the address it gives you (looks like a long string of characters)
  3. Open the file programs/agentescrow/src/lib.rs in a text editor
  4. Find the line that says:

     declare_id!("Escrow11111111111111111111111111111111111111");

  5. Replace it with your real address:

     declare_id!("YOUR_REAL_ADDRESS_HERE");

  6. Save the file
  7. Also update the same address in Anchor.toml where it says:

     agentescrow = "Escrow11111111111111111111111111111111111111"

  8. Replace with your real address there too
  9. Run anchor build again:

     anchor build

================================================================
TROUBLESHOOTING
================================================================

If ANYTHING goes wrong, do this:

1. Copy the full error message
2. Open Claude (claude.ai) or ChatGPT (chatgpt.com)
3. Paste this prompt:

   "I'm setting up a Solana development environment to build a smart contract
   using Anchor framework. I got this error:

   [PASTE YOUR ERROR HERE]

   I'm on [Mac/Windows]. How do I fix this? Give me simple step-by-step
   instructions, I'm not a developer."

4. Follow what the AI tells you
5. If it still doesn't work, paste the new error and ask again

================================================================
WHAT TO DO AFTER SETUP IS COMPLETE
================================================================

Once you can run "anchor test" and see 5 passing tests:

1. Create a GitHub account at github.com (free)
2. Upload your project to GitHub
3. Update your Solana grant application with the GitHub link
4. Submit the grant application at solana.org/grants-funding
5. Join the Solana Discord and share what you've built

================================================================
TIME ESTIMATE
================================================================

Step 1 (Rust):              5 minutes
Step 2 (Solana CLI):        5 minutes
Step 3 (Node.js):           3 minutes
Step 4 (Anchor):            15 minutes (longest step, be patient)
Step 5 (Solana config):     2 minutes
Step 6 (Download project):  2 minutes
Step 7 (Build):             10 minutes
Step 8 (Test):              5 minutes
Step 9 (Update program ID): 5 minutes

TOTAL: About 50-60 minutes

Most of this is waiting for things to install. You just copy, paste, and wait.
