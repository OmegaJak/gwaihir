# <img src="/crates/gwaihir-client/assets/eagle.png" width=50> Gwaihir 
[![dependency status](https://deps.rs/repo/github/omegajak/gwaihir/status.svg)](https://deps.rs/repo/github/omegajak/gwaihir)

A pure-Rust 🦀 desktop app letting trusted friends share their desktop usage with each other to help automatically & remotely answer the question: *“Is now a good time?”*

Its goal is to fill the information gap that exists when collaborating remotely versus in-person. In person, one uses countless cues to determine if a colleague is bored, thinking, in a meeting, etc. When remote, almost all of these cues are missing. Gwaihir aims to automatically transmit just enough information to trusted colleagues/friends so they can naturally reach out when they see that it's a good time.

<img src="https://github.com/OmegaJak/omegajak.github.io/blob/gh-pages/Misc/Gwaihir/ContrivedLOTRExample.png" width=400>

## Current State
Gwaihir is not currently targeting widespread usage, though there's nothing preventing anyone from using it. The main limitation is that it uses the [SpacetimeDB](https://spacetimedb.com/) testnet as an easy way to transmit info from user to user. This requires the creation of your own Spacetime server and inputting its name into Gwaihir. There is no default public server for obvious privacy reasons. Also note that information is not encrypted, so if someone guesses the SpacetimeDB server name, they will be able to see user data.

Currently supports Windows and (mostly) Linux.

## Details
Currently, the following information is shared:
- Online/offline status
- Desktop lock status
- Last update time
- The application name of the currently used desktop application (something like "Google Chrome") and when you started using it
- An overview of which desktop applications have been used in the past 10 minutes and for how long, rounded to the nearest 10-second interval
- Number of keyboard key presses, mouse button presses, and amount of mouse movement in the past 5 minutes
  -  No other details are collected or shared about these - they are immediately quantified into the number of key/button presses or the distance moved by the mouse
  -  The data over the past 5 minutes, grouped into 10-second buckets, is also shared
-  The names of the apps currently using the microphone

Using this information, you can make guesses about what other Gwaihir users are currently doing:
- If you see someone currently using "Visual Studio Code" and furiously typing, you can probably guess that they are actively coding
- If you see someone with no mouse/keyboard activity that's been using "Microsoft Teams" for the past 10 minutes, and that Teams is currently using their microphone, you can guess that they're in a meeting
- If they're locked, they probably stepped away and it might not be a great time to reach out

These are just examples, but the point is that when you work closely with someone Gwaihir can help provide enough information to make guesses about what they're up to and whether it would be a good time to message/call. Humans are pretty good at noticing patterns, and Gwaihir tries to provide enough information to apply those patterns to deciding what a friend is up to.

## A Note on Privacy
Gwaihir's goal is **NOT** to allow employers/managers to spy on their employees, or to facilitate spying on anyone against their will. Explicit measures have been taken in its design to avoid this and others are planned in the future. Gwaihir is for trusted friends/colleagues to stay closer remotely, **NOT** to aid in invading user privacy.

Below, specific measures that have already been taken to aid privacy are described, along with some of the future plans.

### Currently Implemented Privacy-Enforcing Features
- Only the name of the current application is transmitted, no further details like the window title which could betray sensitive information
- The code monitoring mouse/keyboard usage immediately transforms the received event into a purely numeric representation that's added to the rolling total - i.e., a value of 1.0 for each keypress regardless of key, and a value representing moved mouse distance, regardless of positioning
- Users can see exactly what data is being sent in-app
- Data that may involve user path information is stripped, transmitting only the final part of the path
- Gwaihir starts with a very visible window

### Future plans
- Allow users to toggle which 'sensors' are active, perhaps based on who they're being shared with
- Ability to fake the data for any 'sensor', allowing the local user to lie about anything they're sending
- Hopefully move to a purely P2P data transmission architecture, so the data is only sent to the users you share with and is never persisted on an intermediate server
- Encrypt data in transit and require users to explicitly share keys for others to decrypt

## Persistence Locations

| OS      | SpacetimeDB credentials    | Primary persistence & logs                |
| ------- | -------------------------- | ----------------------------------------- |
| Windows | `C:\Users\{USER}\.gwaihir` | `C:\Users\{USER}\AppData\Roaming\gwaihir` |
| Linux   | `~/.gwaihir`               | `~/.local/share/gwaihir`                  | 

## Attributions
- [Eagle icon created by Culmbio - Flaticon](https://www.flaticon.com/free-icons/eagle)
- Test server hosted by [SpacetimeDB](https://spacetimedb.com/)
- GUI built using [egui](https://github.com/emilk/egui)
