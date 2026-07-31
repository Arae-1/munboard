"use strict";

const events = require("../events.js");
const misc = require("../util/misc.js");
const search = require("../util/search.js");
const views = require("../util/views.js");
const CustomFieldAutoCompleteControl = require("../controls/custom_field_auto_complete_control.js");

const template = views.getTemplate("custom-fields-header");

class CustomFieldsHeaderView extends events.EventTarget {
    constructor(ctx) {
        super();

        this._hostNode = ctx.hostNode;
        views.replaceContent(this._hostNode, template(ctx));

        if (this._queryInputNode) {
            this._autoCompleteControl = new CustomFieldAutoCompleteControl(
                this._queryInputNode,
                {
                    confirm: (cf) =>
                        this._autoCompleteControl.replaceSelectedText(
                            misc.escapeSearchTerm(cf.name),
                            true
                        ),
                }
            );
        }

        search.searchInputNodeFocusHelper(this._queryInputNode);

        this._formNode.addEventListener("submit", (e) => this._evtSubmit(e));
    }

    get _formNode() {
        return this._hostNode.querySelector("form");
    }

    get _queryInputNode() {
        return this._hostNode.querySelector("[name=search-text]");
    }

    _evtSubmit(e) {
        e.preventDefault();
        this._queryInputNode.blur();
        this.dispatchEvent(
            new CustomEvent("navigate", {
                detail: {
                    parameters: {
                        query: this._queryInputNode.value,
                        page: 1,
                    },
                },
            })
        );
    }
}

module.exports = CustomFieldsHeaderView;
