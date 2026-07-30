"use strict";

const router = require("../router.js");
const api = require("../api.js");
const uri = require("../util/uri.js");
const misc = require("../util/misc.js");
const CustomFieldList = require("../models/custom_field_list.js");
const topNavigation = require("../models/top_navigation.js");
const CustomFieldsHeaderView = require("../views/custom_fields_header_view.js");
const CustomFieldsPageView = require("../views/custom_fields_page_view.js");
const EmptyView = require("../views/empty_view.js");

const fields = [
    "id",
    "name",
    "description",
    "color",
    "creationTime",
];

class CustomFieldListController {
    constructor(ctx) {
        if (!api.hasPrivilege("custom_field_list")) {
            this._view = new EmptyView();
            this._view.showError("You don't have privileges to view custom fields.");
            return;
        }

        this._ctx = ctx;
        this._allCustomFields = [];

        topNavigation.activate("custom-fields");
        topNavigation.setTitle("Listing custom fields");

        const hostNode = document.getElementById("content-holder");
        hostNode.innerHTML = "";

        this._pageHeaderHolderNode = document.createElement("div");
        this._pageHeaderHolderNode.classList.add("page-header-holder");
        this._pageContentHolderNode = document.createElement("div");
        this._pageContentHolderNode.classList.add("page-content-holder");
        hostNode.appendChild(this._pageHeaderHolderNode);
        hostNode.appendChild(this._pageContentHolderNode);

        this._headerView = new CustomFieldsHeaderView({
            hostNode: this._pageHeaderHolderNode,
            parameters: ctx.parameters,
            canCreate: api.hasPrivilege("custom_field_create"),
        });
        this._headerView.addEventListener("navigate", (e) =>
            this._evtNavigate(e)
        );

        this._loadPage();
    }

    showSuccess(message) {
        const views = require("../util/views.js");
        views.showSuccess(this._pageContentHolderNode, message);
    }

    showError(message) {
        const views = require("../util/views.js");
        views.showError(this._pageContentHolderNode, message);
    }

    _evtNavigate(e) {
        router.showNoDispatch(
            uri.formatClientLink("custom-fields", e.detail.parameters)
        );
        Object.assign(this._ctx.parameters, e.detail.parameters);
        this._renderFiltered();
    }

    _loadPage() {
        CustomFieldList.get(fields).then(
            (response) => {
                this._allCustomFields = response.results;
                this._renderFiltered();
            },
            (error) => {
                this._view = new EmptyView();
                this._view.showError(error.message);
            }
        );
    }

    _renderFiltered() {
        const query = (this._ctx.parameters.query || "").toLowerCase().trim();
        let results = [...this._allCustomFields];

        if (query) {
            const tokens = query.split(/\s+/);
            for (let token of tokens) {
                let negate = false;
                let term = token;
                if (token.startsWith("-")) {
                    negate = true;
                    term = token.slice(1);
                }

                if (term.startsWith("name:")) {
                    const val = term.slice(5);
                    results = results.filter((cf) => {
                        const match = cf.name.toLowerCase().includes(val);
                        return negate ? !match : match;
                    });
                } else if (term.startsWith("color:")) {
                    const val = term.slice(6);
                    results = results.filter((cf) => {
                        const match = cf.color && cf.color.toLowerCase().includes(val);
                        return negate ? !match : match;
                    });
                } else {
                    results = results.filter((cf) => {
                        const match =
                            cf.name.toLowerCase().includes(term) ||
                            (cf.description && cf.description.toLowerCase().includes(term)) ||
                            (cf.color && cf.color.toLowerCase().includes(term));
                        return negate ? !match : match;
                    });
                }
            }
        }

        // Handle sort
        const sortQuery = this._ctx.parameters.query || "";
        if (sortQuery.includes("-sort:name")) {
            results.sort((a, b) => b.name.localeCompare(a.name));
        } else if (sortQuery.includes("-sort:creation-time")) {
            results.sort((a, b) => b.creationTime.localeCompare(a.creationTime));
        } else if (sortQuery.includes("sort:creation-time")) {
            results.sort((a, b) => a.creationTime.localeCompare(b.creationTime));
        } else if (sortQuery.includes("-sort:description")) {
            results.sort((a, b) => (b.description || "").localeCompare(a.description || ""));
        } else if (sortQuery.includes("sort:description")) {
            results.sort((a, b) => (a.description || "").localeCompare(b.description || ""));
        } else if (sortQuery.includes("-sort:color")) {
            results.sort((a, b) => (b.color || "").localeCompare(a.color || ""));
        } else if (sortQuery.includes("sort:color")) {
            results.sort((a, b) => (a.color || "").localeCompare(b.color || ""));
        } else {
            results.sort((a, b) => a.name.localeCompare(b.name));
        }

        this._view = new CustomFieldsPageView({
            hostNode: this._pageContentHolderNode,
            customFields: results,
            parameters: this._ctx.parameters,
        });
    }
}

module.exports = (router) => {
    router.enter(["custom-fields"], (ctx, next) => {
        ctx.controller = new CustomFieldListController(ctx);
    });
};
