"use strict";

const events = require("../events.js");
const misc = require("../util/misc.js");
const uri = require("../util/uri.js");
const views = require("../util/views.js");
const PoolAutoCompleteControl = require("./pool_auto_complete_control.js");

const template = views.getTemplate("pool-input");

class PoolChildrenInputControl extends events.EventTarget {
    constructor(hostNode, pool) {
        super();
        this._hostNode = hostNode;
        this._children = [...pool.children];
        this._nodes = new Map();
        this._editAreaNode = template();
        this._inputNode = this._editAreaNode.querySelector("input");
        this._listNode = this._editAreaNode.querySelector("ul");
        this._autoCompleteControl = new PoolAutoCompleteControl(this._inputNode, {
            getTextToFind: () => this._inputNode.value,
            filterPool: (candidate) => candidate.id === pool.id || this._children.some((child) => child.id === candidate.id),
            confirm: (candidate) => {
                this._inputNode.value = "";
                this._syncValidity();
                this._add(candidate);
            },
            verticalShift: -2,
        });
        hostNode.style.display = "none";
        hostNode.parentNode.insertBefore(this._editAreaNode, hostNode.nextSibling);
        for (let child of this._children) {
            this._listNode.appendChild(this._createNode(child));
        }
        this._inputNode.addEventListener("input", () => this._syncValidity());
        this._errorNode = document.createElement("p");
        this._errorNode.classList.add("hint");
        this._errorNode.setAttribute("aria-live", "polite");
        hostNode.parentNode.appendChild(this._errorNode);
        this._inputNode.id = "child-pools-picker";
        this._inputNode.setAttribute("aria-describedby", "child-pools-hint");
        const labelNode = hostNode.parentNode.querySelector("label");
        if (labelNode) {
            labelNode.setAttribute("for", this._inputNode.id);
        }
        this._inputNode.addEventListener("invalid", () => {
            this._errorNode.textContent = this._inputNode.validationMessage;
        });
        this._inputNode.addEventListener("keydown", (e) => {
            if (e.key === "Enter" && this._inputNode.value.trim()) {
                e.preventDefault();
                this.reportValidity();
            }
        });
    }

    getIds() {
        return this._children.map((child) => child.id);
    }

    reportValidity() {
        this._syncValidity();
        if (this._inputNode.validity.valid) {
            return true;
        }
        this._inputNode.focus();
        this._inputNode.reportValidity();
        return false;
    }

    _syncValidity() {
        const message = this._inputNode.value.trim() ? "Select a pool from the suggestion list. Clear the field to save without adding a pool." : "";
        this._inputNode.setCustomValidity(message);
        if (!message) {
            this._errorNode.textContent = "";
        }
    }

    _add(pool) {
        if (this._children.some((child) => child.id === pool.id)) {
            return;
        }
        const child = { id: pool.id, name: pool.names[0], category: pool.category, postCount: pool.postCount || 0 };
        this._children.push(child);
        this._listNode.appendChild(this._createNode(child));
        this.dispatchEvent(new CustomEvent("change"));
    }

    _remove(child) {
        this._children.splice(this._children.indexOf(child), 1);
        this._nodes.get(child.id).remove();
        this._nodes.delete(child.id);
        this.dispatchEvent(new CustomEvent("change"));
    }

    _move(child, delta) {
        const oldIndex = this._children.indexOf(child);
        const newIndex = oldIndex + delta;
        if (newIndex < 0 || newIndex >= this._children.length) {
            return;
        }
        this._children.splice(oldIndex, 1);
        this._children.splice(newIndex, 0, child);
        const node = this._nodes.get(child.id);
        this._listNode.insertBefore(node, delta < 0 ? node.previousSibling : node.nextSibling.nextSibling);
        this.dispatchEvent(new CustomEvent("change"));
    }

    _createNode(child) {
        const node = document.createElement("li");
        for (let [className, text, label, handler] of [
            ["remove-pool", "×", "Remove " + (child.name || "child pool"), () => this._remove(child)],
            ["move-up", "↑", "Move " + (child.name || "child pool") + " up", () => this._move(child, -1)],
            ["move-down", "↓", "Move " + (child.name || "child pool") + " down", () => this._move(child, 1)],
        ]) {
            const action = document.createElement("a");
            action.classList.add(className);
            action.href = "";
            action.textContent = text;
            action.title = label;
            action.setAttribute("aria-label", label);
            action.addEventListener("click", (e) => { e.preventDefault(); handler(); });
            node.appendChild(action);
        }
        const link = document.createElement("a");
        link.classList.add(misc.makeCssName(child.category, "pool"));
        link.href = uri.formatClientLink("pool", child.id);
        link.textContent = child.name || "pool " + child.id;
        node.appendChild(link);
        this._nodes.set(child.id, node);
        return node;
    }
}

module.exports = PoolChildrenInputControl;
