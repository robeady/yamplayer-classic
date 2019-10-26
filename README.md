# Yamplayer

A music player written in rust and typescript.

## Backend

This is a prototype music player server

### Installation

Use rustup to install the nightly edition of the rust toolchain and set it as default in the server directory:

    rustup toolchain install nightly
    rustup override set nightly
        
Then use cargo to build the server project.

## Frontend

This is a create-react-app webapp that interacts with the music player server.

### Installation

Install node and pnpm.

`pnpm install` in the frontend directory

### Running

`pnpm start` to run in dev mode.

You might want to open at 127.0.0.1 rather than localhost to avoid random 300ms delays in requests to the server due to
some chrome bug

### Dev setup

I use vscode.

It has built in typescript support; I select the workspace version of typescript by clicking the version number in the bottom right when editing a TS file.

I use the prettier extension and enable autoformatting on save in this project.

I use the eslint extension. It must be configured to lint typescript files by adding this to your settings.json:

    "eslint.validate": ["javascript", "javascriptreact", "typescript", "typescriptreact"]

I also use the vscode-styled-components extension for syntax highlighting and autocomplete in `css` blocks.



    
