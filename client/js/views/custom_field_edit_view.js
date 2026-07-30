"use strict";

const events = require("../events.js");
const views = require("../util/views.js");

const template = views.getTemplate("custom-field-edit");

class CustomFieldEditView extends events.EventTarget {
    constructor(ctx) {
        super();

        this._customField = ctx.customField;
        this._hostNode = ctx.hostNode;
        views.replaceContent(this._hostNode, template(ctx));

        views.decorateValidator(this._formNode);

        for (let node of this._formNode.querySelectorAll(
            "input, select, textarea"
        )) {
            node.addEventListener("change", (e) => {
                this.dispatchEvent(new CustomEvent("change"));
            });
        }

        this._formNode.addEventListener("submit", (e) => this._evtSubmit(e));
    }

    clearMessages() {
        views.clearMessages(this._hostNode);
    }

    enableForm() {
        views.enableForm(this._formNode);
    }

    disableForm() {
        views.disableForm(this._formNode);
    }

    showSuccess(message) {
        views.showSuccess(this._hostNode, message);
    }

    showError(message) {
        views.showError(this._hostNode, message);
    }

    _evtSubmit(e) {
        e.preventDefault();
        this.dispatchEvent(
            new CustomEvent("submit", {
                detail: {
                    customField: this._customField,

                    name: this._nameFieldNode
                        ? this._nameFieldNode.value
                        : undefined,

                    description: this._descriptionFieldNode
                        ? this._descriptionFieldNode.value
                        : undefined,

                    color: this._colorFieldNode
                        ? this._colorFieldNode.value
                        : undefined,
                },
            })
        );
    }

    get _formNode() {
        return this._hostNode.querySelector("form");
    }

    get _nameFieldNode() {
        return this._formNode.querySelector(".name input");
    }

    get _descriptionFieldNode() {
        return this._formNode.querySelector(".description textarea");
    }

    get _colorFieldNode() {
        return this._formNode.querySelector(".color input");
    }
}

module.exports = CustomFieldEditView;
