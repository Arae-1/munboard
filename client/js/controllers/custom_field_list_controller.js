"use strict";

const router = require("../router.js");
const api = require("../api.js");
const uri = require("../util/uri.js");
const CustomFieldList = require("../models/custom_field_list.js");
const topNavigation = require("../models/top_navigation.js");
const PageController = require("../controllers/page_controller.js");
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
        this._pageController = new PageController();

        if (!api.hasPrivilege("custom_field_list")) {
            this._view = new EmptyView();
            this._view.showError("You don't have privileges to view custom fields.");
            return;
        }

        this._ctx = ctx;
        this._allCustomFields = null;

        topNavigation.activate("custom-fields");
        topNavigation.setTitle("Listing custom fields");

        this._headerView = new CustomFieldsHeaderView({
            hostNode: this._pageController.view.pageHeaderHolderNode,
            parameters: ctx.parameters,
            canCreate: api.hasPrivilege("custom_field_create"),
        });
        this._headerView.addEventListener("navigate", (e) =>
            this._evtNavigate(e)
        );

        this._syncPageController();
    }

    showSuccess(message) {
        this._pageController.showSuccess(message);
    }

    showError(message) {
        this._pageController.showError(message);
    }

    _evtNavigate(e) {
        router.showNoDispatch(
            uri.formatClientLink("custom-fields", e.detail.parameters)
        );
        Object.assign(this._ctx.parameters, e.detail.parameters);
        this._syncPageController();
    }

    _filterSort(results, query) {
        let filtered = [...results];
        const q = (query || "").trim();

        if (q) {
            const tokens = q.split(/\s+/).filter((t) => !t.startsWith("sort:") && !t.startsWith("-sort:"));
            for (let token of tokens) {
                let negate = false;
                let term = token;
                if (token.startsWith("-")) {
                    negate = true;
                    term = token.slice(1);
                }

                if (term.startsWith("name:")) {
                    const val = term.slice(5);
                    filtered = filtered.filter((cf) => {
                        const match = cf.name.toLowerCase().includes(val);
                        return negate ? !match : match;
                    });
                } else if (term.startsWith("color:")) {
                    const val = term.slice(6);
                    filtered = filtered.filter((cf) => {
                        const match = cf.color && cf.color.toLowerCase().includes(val);
                        return negate ? !match : match;
                    });
                } else {
                    filtered = filtered.filter((cf) => {
                        const match =
                            cf.name.toLowerCase().includes(term) ||
                            (cf.description && cf.description.toLowerCase().includes(term)) ||
                            (cf.color && cf.color.toLowerCase().includes(term));
                        return negate ? !match : match;
                    });
                }
            }
        }

        if (q.includes("-sort:name")) {
            filtered.sort((a, b) => b.name.localeCompare(a.name));
        } else if (q.includes("-sort:creation-time")) {
            filtered.sort((a, b) => b.creationTime.localeCompare(a.creationTime));
        } else if (q.includes("sort:creation-time")) {
            filtered.sort((a, b) => a.creationTime.localeCompare(b.creationTime));
        } else if (q.includes("-sort:description")) {
            filtered.sort((a, b) => (b.description || "").localeCompare(a.description || ""));
        } else if (q.includes("sort:description")) {
            filtered.sort((a, b) => (a.description || "").localeCompare(b.description || ""));
        } else {
            filtered.sort((a, b) => a.name.localeCompare(b.name));
        }

        return filtered;
    }

    _syncPageController() {
        this._pageController.run({
            parameters: this._ctx.parameters,
            defaultLimit: 50,
            getClientUrlForPage: (offset, limit) => {
                const parameters = Object.assign({}, this._ctx.parameters, {
                    offset: offset,
                    limit: limit,
                });
                return uri.formatClientLink("custom-fields", parameters);
            },
            requestPage: (offset, limit) => {
                const fetchPromise = this._allCustomFields
                    ? Promise.resolve(this._allCustomFields)
                    : CustomFieldList.get(fields).then((response) => {
                          this._allCustomFields = response.results;
                          return this._allCustomFields;
                      });

                return fetchPromise.then((results) => {
                    const filtered = this._filterSort(
                        results,
                        this._ctx.parameters.query
                    );
                    return Promise.resolve({
                        results: filtered,
                        total: filtered.length,
                        offset: 0,
                        limit: filtered.length,
                    });
                });
            },
            pageRenderer: (pageCtx) => {
                return new CustomFieldsPageView(pageCtx);
            },
        });
    }
}

module.exports = (router) => {
    router.enter(["custom-fields"], (ctx, next) => {
        ctx.controller = new CustomFieldListController(ctx);
    });
};
