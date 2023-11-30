# Chrome extension for generating a random playlist from your library tracks, albums and playlists 

This extension builds a Spotify playlist out of random tracks found in _My Library_ to let you listen to everything you have in there on shuffle.  

It picks tracks from Liked Songs, Liked Albums and Liked Playlists.

* Install the extension
* Log into Spotify
* Create a new playlist
* Click on the extension button in the toolbar menu to add tracks

Once the extension is running, you can close the Spotify tab or navigate away from the playlist. It will continue creating a shuffled sample of your library in the background. 

The extension popup window receives updates from the background script working on the task. Keep it open to watch the progress.

### Feedback and bug reports

This project is under active development. Feel free to [open an issue](https://github.com/rimutaka/spotify-playlist-builder/issues) if you run into problems or have any kind of feedback.

### Privacy policy

This extension does not collect any information about the user or user activities.  
The only site it communicates with is Spotify.

## Under the hood

The plugin intercepts the session token from Spotify requests and impersonates the Spotify client to:
* read the contents of the user library
* add random tracks to the current playlist

It does not transmit any of your data to any third party. All requests go to Spotify.

Most of the work is done by [a WASM module](wasm_mod) built in Rust.

## Attributions

The extension stub was taken from https://github.com/theberrigan/rust-wasm-chrome-ext by https://github.com/theberrigan.

The toolbar icon is based on a vector image by https://rawpixel.com.