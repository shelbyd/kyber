const React = require('react');
const ReactDOM = require('react-dom/client');
const htm = require('htm');

const html = htm.bind(React.createElement);

module.exports = {
  React,
  ReactDOM,
  html,
};
