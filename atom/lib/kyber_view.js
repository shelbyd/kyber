"use babel";

const { ReactDOM, html } = require("./deps.js");
const { useState, useEffect } = require("react");

export default class KyberView {
  constructor(serializedState) {
    this.element = document.createElement("div");
    this.element.id = "kyber-view";

    this.root = ReactDOM.createRoot(this.element);
    this.root.render(html`<${App} />`);
  }

  serialize() {}

  destroy() {
    this.root.unmount();
    this.element.remove();
  }

  getElement() {
    return this.element;
  }
}

function App() {
  const [counter, setCounter] = useState(0);

  useEffect(() => {
    atom.workspace.getActiveTextEditor().onDidChangeCursorPosition(event => {
      console.log({event});
    });
  }, []);

  return html`<div class="message">The Kyber package is Alive! It's ALIVE! ${counter}</div>
    <button onClick=${() => setCounter(counter + 1)}>
      Increment
    </button>`;
}
