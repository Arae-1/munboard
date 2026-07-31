"use strict";

const api = require("../api.js");
const events = require("../events.js");
const views = require("../util/views.js");
const CustomFieldAutoCompleteControl = require("./custom_field_auto_complete_control.js");

const template = views.getTemplate("custom-field-input");

class CustomFieldInputControl extends events.EventTarget {
    constructor(hostNode, customFields, customFieldColors) {
        super();
        this._customFields = customFields;
        this._customFieldColors = customFieldColors || {};
        this._selectedColor = null;
        this._editingName = null;
        this._hostNode = hostNode;

        this._editAreaNode = template();
        this._nameInputNode = this._editAreaNode.querySelector("input.cf-name");
        this._valueInputNode = this._editAreaNode.querySelector("input.cf-value");
        this._addButtonNode = this._editAreaNode.querySelector("button");
        this._listNode = this._editAreaNode.querySelector(
            "ul.compact-custom-fields"
        );

        this._addButtonNode.textContent = "Add";

        this._autoCompleteControl = new CustomFieldAutoCompleteControl(
            this._nameInputNode,
            {
                getTextToFind: () => {
                    return this._nameInputNode.value;
                },
                confirm: (cf) => {
                    this._nameInputNode.value = cf.name;
                    this._selectedColor = cf.color;
                },
                verticalShift: -2,
            }
        );

        this._hostNode.style.display = "none";
        this._hostNode.parentNode.insertBefore(
            this._editAreaNode,
            hostNode.nextSibling
        );

        // add existing custom fields
        for (let name of Object.keys(this._customFields).sort()) {
            const listItemNode = this._createListItemNode(
                name,
                this._customFields[name],
                this._customFieldColors[name]
            );
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
        this._nameInputNode.addEventListener("input", (e) => {
            if (!this._editingName) {
                return;
            }
            if (this._nameInputNode.value.trim() !== this._editingName) {
                this._exitEditMode();
            }
        });
    }

    _evtAddClick(e) {
        e.preventDefault();
        this._addCurrent();
    }

    _evtEditClick(name) {
        this._editingName = name;
        this._nameInputNode.value = name;
        this._valueInputNode.value = this._customFields[name];
        this._addButtonNode.textContent = "Update";
        this._highlightRow(name);
        this._valueInputNode.focus();
    }

    _exitEditMode() {
        this._editingName = null;
        this._nameInputNode.value = "";
        this._valueInputNode.value = "";
        this._addButtonNode.textContent = "Add";
        this._clearHighlight();
    }

    _addCurrent() {
        const name = this._nameInputNode.value.trim();
        const value = parseInt(this._valueInputNode.value, 10);

        if (!name || isNaN(value)) {
            return;
        }

        const color = this._selectedColor;
        this._selectedColor = null;

        if (Object.prototype.hasOwnProperty.call(this._customFields, name)) {
            this._updateListItemNode(name, value);
            this._customFieldColors[name] = color || this._customFieldColors[name];
        } else {
            const listItemNode = this._createListItemNode(name, value, color);
            this._listNode.appendChild(listItemNode);
        }

        this._customFields[name] = value;

        this._exitEditMode();
        this._nameInputNode.focus();

        this.dispatchEvent(new CustomEvent("change"));
    }

    _removeField(name) {
        if (this._editingName === name) {
            this._exitEditMode();
        }
        delete this._customFields[name];
        delete this._customFieldColors[name];
        this._deleteListItemNode(name);
        this.dispatchEvent(new CustomEvent("change"));
    }

    _createListItemNode(name, value, color) {
        const removalLinkNode = document.createElement("a");
        removalLinkNode.classList.add("remove-custom-field");
        removalLinkNode.setAttribute("href", "");
        removalLinkNode.setAttribute("data-pseudo-content", "\u00d7");
        removalLinkNode.addEventListener("click", (e) => {
            e.preventDefault();
            e.stopPropagation();
            this._removeField(name);
        });

        const nameNode = document.createElement("span");
        nameNode.classList.add("custom-field-name");
        nameNode.textContent = name + ":";
        nameNode.style.color = color || api.getCustomFieldDefaultColor();

        const valueNode = document.createElement("span");
        valueNode.classList.add("custom-field-usages");
        valueNode.setAttribute("data-pseudo-content", value);

        const listItemNode = document.createElement("li");
        listItemNode.setAttribute("data-name", name);
        listItemNode.appendChild(removalLinkNode);
        listItemNode.appendChild(nameNode);
        listItemNode.appendChild(valueNode);
        listItemNode.addEventListener("click", (e) => {
            this._evtEditClick(name);
        });

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

    _highlightRow(name) {
        this._clearHighlight();
        const row = this._listNode.querySelector(`li[data-name="${CSS.escape(name)}"]`);
        if (row) {
            row.classList.add("editing");
        }
    }

    _clearHighlight() {
        const active = this._listNode.querySelector("li.editing");
        if (active) {
            active.classList.remove("editing");
        }
    }
}

module.exports = CustomFieldInputControl;
