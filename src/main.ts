import { mount } from 'svelte';
import '../design/tokens.css';
import './app.css';
import './controls.css';
import App from './App.svelte';
import NoteWindow from './NoteWindow.svelte';

// The island window loads index.html; an "Open full" note window loads index.html#note=<path>.
const notePath = new URLSearchParams(location.hash.slice(1)).get('note');
const target = document.getElementById('app')!;

const app = notePath ? mount(NoteWindow, { target, props: { path: notePath } }) : mount(App, { target });

export default app;
