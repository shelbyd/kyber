"use babel";

import KyberView from "./kyber_view";
import { CompositeDisposable } from "atom";

export default {
  kyberView: null,
  panel: null,
  subscriptions: null,

  activate(state) {
    this.kyberView = new KyberView(state.kyberViewState);

    this.panel = atom.workspace.addRightPanel({
      item: this.kyberView.getElement(),
      visible: false,
    });

    this.subscriptions = new CompositeDisposable();

    this.subscriptions.add(
      atom.commands.add("atom-workspace", {
        "kyber:toggle": () => this.toggle(),
      })
    );
  },

  deactivate() {
    this.panel.destroy();
    this.subscriptions.dispose();
    this.kyberView.destroy();
  },

  serialize() {
    return {
      kyberViewState: this.kyberView.serialize(),
    };
  },

  toggle() {
    return this.panel.isVisible()
      ? this.panel.hide()
      : this.panel.show();
  },
};
