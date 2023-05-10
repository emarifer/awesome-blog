# Awesome Blog (Axum/Rust Server)

### Server application to display static assets (a blog web site) and that connects to a remote database (Supabase), developed with Rust.

First, you must create an environment variable (.env) file that contains your Supabase credentials. The file configuration should be as follows:

```bash
SUPABASE_URL=xxxx
SUPABASE_ANON_KEY=xxxx
```

To run the application (in development mode):

```bash
$ cargo run # or cargo watch -q -c -w ./ -x run (cargo-watch must be installed on the system)
```

To build the project for production and minimize its size:

```bash
$ cargo build --release
```

Runs the app in the development mode.<br>
Open [http://localhost:4000](http://localhost:4000) to view it in the browser.

The page will reload if you make edits.<br>

If you edit the project by modifying the styles and/or the Html of the templates, you must have a terminal open in the /awesome-blog/tailwind/ folder by executing this command para regenerar el index.css creado por Tailwindcss:

```bash
$ npm run watch-css
```

To push to production, it is possible to minify the CSS created by Tailwindcss with this command:

```bash
$ npm run build-css-prod
```
