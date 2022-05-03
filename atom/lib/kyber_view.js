'use babel';

const { ReactDOM, html } = require('./deps.js');

export default class KyberView {
  constructor(serializedState) {
    this.element = document.createElement('div');
    this.element.classList.add('kyber');

    ReactDOM.render(
      html`<div class="message">The Kyber package is Alive! It's ALIVE!</div>`,
      this.element,
    );
  }

  serialize() {}

  destroy() {
    this.element.remove();
  }

  getElement() {
    return this.element;
  }
}
