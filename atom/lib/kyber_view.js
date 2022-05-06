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

let performingMutations = false;

const Container = styled.div`
  display: flex;
  flex-direction: column;
  align-items: stretch;

  width: 320px;

  .empty {
    padding: 8px;
  }
`;

function App() {
  const [suggestions, setSuggestions] = useState([]);

  const updateSuggestions = async () => {
    if (performingMutations) return;
    const context = await buildKyberContext();
    const suggestions = await retrieveSuggestions(context);
    setSuggestions(suggestions);
  };

  useEffect(() => {
    const editor = atom.workspace.getActiveTextEditor();

    editor.onDidChangeCursorPosition(updateSuggestions);
    editor.onDidChange(updateSuggestions);
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
  return (await doRpc("suggest", { context })).suggestions;
}

async function doRpc(rpc, input) {
  const timer = `rpc.${rpc}`;
  console.time(timer);

  const kyberCommand = atom.config.get("kyber.kyberCliPath");

  const { stdout, stderr } = await execCommand(`${kyberCommand} rpc ${rpc}`, JSON.stringify(input));

  if (stderr != "") {
    console.log(stderr);
  }

  console.timeEnd(timer);
  return JSON.parse(stdout);
}

async function execCommand(command, stdin) {
  const {exec} = require("child_process");

  const child = exec(command);

  return await new Promise((resolve, reject) => {
    let stdout = "";
    child.stdout.on('data', (s) => {
      stdout += s;
    });

    let stderr = "";
    child.stderr.on('data', (s) => {
      stderr += s;
    });

    child.on('exit', (code, signal) => {
      if (code == 0) {
        resolve({stdout, stderr});
      } else {
        reject(stderr);
      }
    });

    child.stdin.write(stdin);
    child.stdin.end();

    // child.stdin won't actually write without spawning another process, so we spawn the simplest one.
    exec('echo "ffs Node"');
  });
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

async function applySuggestion(suggestion) {
  const context = await buildKyberContext();
  const result = await doRpc("perform", { context, id: suggestion.id });

  const editor = atom.workspace.getActiveTextEditor();
  editor.transact(0, () => {
    performingMutations = true;
    for (const mutation of result.mutations) {
      if (mutation.delete != null) {
        times(mutation.delete, () => editor.delete());
      } else if (mutation.backspace != null) {
        times(mutation.backspace, () => editor.backspace());
      } else if (mutation.insert != null) {
        editor.insertText(mutation.insert);
      } else {
        console.error({mutation});
        throw new Error(`Unrecognized mutation`);
      }
    }
    performingMutations = false;
  });
}

function times(count, fn) {
  for (let i = 0; i < count; i++) {
    fn();
  }
}
