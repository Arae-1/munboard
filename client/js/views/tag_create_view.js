"use strict";

const events = require("../events.js");
const api = require("../api.js");
const misc = require("../util/misc.js");
const views = require("../util/views.js");
const TagInputControl = require("../controls/tag_input_control.js");
const TagList = require("../models/tag_list.js");

const template = views.getTemplate("tag-create");

class TagCreateView extends events.EventTarget {
    constructor(ctx) {
        super();

        this._hostNode = document.getElementById("content-holder");
        views.replaceContent(this._hostNode, template(ctx));

        views.decorateValidator(this._formNode);

        if (this._namesFieldNode) {
            this._namesFieldNode.addEventListener("input", (e) =>
                this._evtNameInput(e)
            );
        }

        if (this._implicationsFieldNode) {
            this._implications = new TagList();
            new TagInputControl(this._implicationsFieldNode, this._implications);
        }
        if (this._suggestionsFieldNode) {
            this._suggestions = new TagList();
            new TagInputControl(this._suggestionsFieldNode, this._suggestions);
        }

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

    _evtNameInput(e) {
        const regex = new RegExp(api.getTagNameRegex());
        const list = misc.splitByWhitespace(this._namesFieldNode.value);

        if (!list.length) {
            this._namesFieldNode.setCustomValidity(
                "Tags must have at least one name."
            );
            return;
        }

        for (let item of list) {
            if (!regex.test(item)) {
                this._namesFieldNode.setCustomValidity(
                    `Tag name "${item}" contains invalid symbols.`
                );
                return;
            }
        }

        this._namesFieldNode.setCustomValidity("");
    }

    _evtSubmit(e) {
        e.preventDefault();
        this.dispatchEvent(
            new CustomEvent("submit", {
                detail: {
                    names: misc.splitByWhitespace(this._namesFieldNode.value),
                    category: this._categoryFieldNode.value,
                    implications: this._implications
                        ? [...this._implications].map((tag) => tag.names[0])
                        : undefined,
                    suggestions: this._suggestions
                        ? [...this._suggestions].map((tag) => tag.names[0])
                        : undefined,
                    description: this._descriptionFieldNode.value,
                },
            })
        );
    }

    get _formNode() {
        return this._hostNode.querySelector("form");
    }

    get _namesFieldNode() {
        return this._formNode.querySelector(".names input");
    }

    get _categoryFieldNode() {
        return this._formNode.querySelector(".category select");
    }

    get _implicationsFieldNode() {
        return this._formNode.querySelector(".implications input");
    }

    get _suggestionsFieldNode() {
        return this._formNode.querySelector(".suggestions input");
    }

    get _descriptionFieldNode() {
        return this._formNode.querySelector(".description textarea");
    }
}

module.exports = TagCreateView;
