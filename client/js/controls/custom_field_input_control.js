"use strict";

const api = require("../api.js");
const misc = require("../util/misc.js");
const events = require("../events.js");
const views = require("../util/views.js");
const CustomFieldAutoCompleteControl = require("./custom_field_auto_complete_control.js");

const template = views.getTemplate("custom-field-input");

class CustomFieldInputControl extends events.EventTarget {
    constructor(hostNode, customFields) {
        super();
        this._customFields = customFields;
        this._hostNode = hostNode;

        // dom
        const editAreaNode = template();
        this._editAreaNode = editAreaNode;
        this._nameInputNode = editAreaNode.querySelector("input.cf-name");
        this._valueInputNode = editAreaNode.querySelector("input.cf-value");
        this._addButtonNode = editAreaNode.querySelector("button");
        this._listNode = editAreaNode.querySelector("ul.compact-custom-fields");

        this._autoCompleteControl = new CustomFieldAutoCompleteControl(
            this._nameInputNode,
            {
                getTextToFind: () => {
                    return this._nameInputNode.value;
                },
                confirm: (cf) => {
                    this._nameInputNode.value = cf.name;
                },
                verticalShift: -2,
            }
        );

        // show
        this._hostNode.style.display = "none";
        this._hostNode.parentNode.insertBefore(
            this._editAreaNode,
            hostNode.nextSibling
        );

        // add existing custom fields
        for (let name of Object.keys(this._customFields)) {
            const listItemNode = this._createListItemNode(name, this._customFields[name]);
            this._listNode.appendChild(listItemNode);
        }

        this._addButtonNode.addEventListener("click", (e) => this._evtAddClick(e));
        this._nameInputNode.addEventListener("keydown", (e) => {
            if (e.key === "Enter") {
                e.preventDefault();
                this._addCurrent();
            }
        });
        this._valueInputNode.addEventListener("keydown", (e) => {
            if (e.key === "Enter") {
                e.preventDefault();
                this._addCurrent();
            }
        });
    }

    _evtAddClick(e) {
        e.preventDefault();
        this._addCurrent();
    }

    _addCurrent() {
        const name = this._nameInputNode.value.trim();
        const value = parseInt(this._valueInputNode.value, 10);

        if (!name || isNaN(value)) {
            return;
        }

        if (Object.prototype.hasOwnProperty.call(this._customFields, name)) {
            this._updateListItemNode(name, value);
        } else {
            const listItemNode = this._createListItemNode(name, value);
            this._listNode.appendChild(listItemNode);
        }

        this._customFields[name] = value;

        this._nameInputNode.value = "";
        this._valueInputNode.value = "";
        this._nameInputNode.focus();

        this.dispatchEvent(new CustomEvent("change"));
    }

    _removeField(name) {
        delete this._customFields[name];
        this._deleteListItemNode(name);
        this.dispatchEvent(new CustomEvent("change"));
    }

    _createListItemNode(name, value) {
        const removalLinkNode = document.createElement("a");
        removalLinkNode.classList.add("remove-custom-field");
        removalLinkNode.setAttribute("href", "");
        removalLinkNode.setAttribute("data-pseudo-content", "×");
        removalLinkNode.addEventListener("click", (e) => {
            e.preventDefault();
            this._removeField(name);
        });

        const nameNode = document.createElement("span");
        nameNode.classList.add("custom-field-name");
        nameNode.textContent = name;

        const valueNode = document.createElement("span");
        valueNode.classList.add("custom-field-usages");
        valueNode.setAttribute("data-pseudo-content", value);

        const listItemNode = document.createElement("li");
        listItemNode.setAttribute("data-name", name);
        listItemNode.appendChild(removalLinkNode);
        listItemNode.appendChild(nameNode);
        listItemNode.appendChild(valueNode);

        return listItemNode;
    }

    _updateListItemNode(name, value) {
        const listItemNode = this._listNode.querySelector(`li[data-name="${CSS.escape(name)}"]`);
        if (listItemNode) {
        const valueNode = listItemNode.querySelector(".custom-field-usages");
        if (valueNode) {
            valueNode.setAttribute("data-pseudo-content", value);
            }
        }
    }

    _deleteListItemNode(name) {
        const listItemNode = this._listNode.querySelector(`li[data-name="${CSS.escape(name)}"]`);
        if (listItemNode) {
            listItemNode.parentNode.removeChild(listItemNode);
        }
    }
}

module.exports = CustomFieldInputControl;
