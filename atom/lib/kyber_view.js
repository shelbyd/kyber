"use babel";

const { ReactDOM, html, styled, useState, useEffect } = require("./deps.js");

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

const Container = styled.div`
  display: flex;
  flex-direction: column;
  align-items: stretch;

  .empty {
    padding: 8px;
  }
`;

function App() {
  const [suggestions, setSuggestions] = useState([]);

  const updateSuggestions = async () => {
    const context = await buildKyberContext();
    const suggestions = await retrieveSuggestions(context);
    console.log({suggestions});
    setSuggestions(suggestions);
  };

  useEffect(() => {
    const editor = atom.workspace.getActiveTextEditor();

    editor.onDidChangeCursorPosition(updateSuggestions);

    updateSuggestions();
  }, []);

  return html`<${Container}>
    ${
      suggestions.length > 0
        ? suggestions.map((s) => html`<${Suggestion} suggestion=${s} />`)
        : html`<p class="empty">
            No suggested refactorings for the current position
          </p>`
    }
  </${Container}>`;
}

async function buildKyberContext() {
  const editor = atom.workspace.getActiveTextEditor();

  const range = editor.getSelectedBufferRange();
  const buffer = editor.getBuffer();
  const start = buffer.characterIndexForPosition(range.start);
  const end = buffer.characterIndexForPosition(range.end);

  const text = buffer.getText();
  const pre = text.substr(0, start);
  const selected = text.substr(start, end - start);
  const post = text.substr(end, text.length - end);

  return {
    contents: [
      { text: pre, selected: false },
      { text: selected, selected: true },
      { text: post, selected: false },
    ],
  };
}

async function retrieveSuggestions(context) {
  const util = require('util');
  const exec = util.promisify(require('child_process').exec);

  const kyberCommand = atom.config.get('kyber.kyberCliPath');
  const promise = exec(`${kyberCommand} rpc suggest`);

  promise.child.stdin.write(JSON.stringify({context}));
  promise.child.stdin.end();

  const { stdout, stderr } = await promise;

  if (stderr != '') {
    console.log(stderr);
  }

  return JSON.parse(stdout).suggestions;
}

const SuggestionContainer = styled.div`
  padding: 8px;
  cursor: pointer;

  &:hover {
    background-color: rgba(0, 0, 0, 0.5);
  }
`;

function Suggestion({ suggestion }) {
  return html`
    <${SuggestionContainer} onClick=${() => applySuggestion(suggestion)}>
      <h1>${suggestion.name}</h1>
      <p>${suggestion.description}</p>
    </${SuggestionContainer}>
  `;
}

function applySuggestion(suggestion) {
  return;

  // Will use code like the following:
  // const editor = atom.workspace.getActiveTextEditor();
  // editor.transact(0, () => {
  //   editor.backspace();
  //   editor.delete();
  //   editor.insertText("==");
  // });
}
