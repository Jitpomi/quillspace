/**
 * WHAT IS THIS FILE?
 *
 * The entry.client.tsx file is the entry point for the browser. This file is used to
 * hydrate the application in the browser.
 *
 * Feel free to modify this file, but don't remove the default export.
 */
import { render } from '@builder.io/qwik';
import Root from './root';

render(document, <Root />);
