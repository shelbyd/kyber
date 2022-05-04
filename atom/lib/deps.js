const React = require("react");
const { useState, useEffect } = require("react");
const ReactDOM = require("react-dom/client");
const htm = require("htm");
const styled = require("styled-components");

const html = htm.bind(React.createElement);

module.exports = {
  React,
  ReactDOM,
  html,
  styled: styled.default,
  useState,
  useEffect,
};
