"use strict";

const misc = require("../util/misc.js");
const CustomFieldList = require("../models/custom_field_list.js");
const AutoCompleteControl = require("./auto_complete_control.js");

let _fieldNamesCache = null;
let _fieldNamesPromise = null;

function _loadFieldNames() {
    if (_fieldNamesCache !== null) {
        return Promise.resolve(_fieldNamesCache);
    }
    if (_fieldNamesPromise !== null) {
        return _fieldNamesPromise;
    }
    _fieldNamesPromise = CustomFieldList.get(["name"]).then((response) => {
        _fieldNamesCache = new Set(
            response.results.map((cf) => cf.name.toLowerCase())
        );
        return _fieldNamesCache;
    });
    return _fieldNamesPromise;
}

function isValidFieldName(name) {
    return _fieldNamesCache !== null && _fieldNamesCache.has(name.toLowerCase());
}

function ensureFieldNamesLoaded() {
    return _loadFieldNames();
}

class CustomFieldAutoCompleteControl extends AutoCompleteControl {
    constructor(input, options) {
        options.getMatches = (text) => {
            const term = misc.escapeSearchTerm(text);

            return new Promise((resolve, reject) => {
                CustomFieldList.get(["id", "name", "description", "color"]).then(
                    (response) => {
                        const matches = response.results
                            .filter((cf) =>
                                cf.name.toLowerCase().includes(text.toLowerCase())
                            )
                            .slice(0, this._options.maxResults)
                            .map((cf) => {
                                const regex = new RegExp(
                                    "(" + misc.escapeSearchTerm(text) + ")",
                                    "gi"
                                );
                                const caption = misc
                                    .escapeHtml(cf.name)
                                    .replace(regex, "<b>$1</b>");
                                return {
                                    caption: caption,
                                    value: cf,
                                };
                            });
                        resolve(matches);
                    },
                    reject
                );
            });
        };

        super(input, options);
    }
}

module.exports = CustomFieldAutoCompleteControl;
module.exports.isValidFieldName = isValidFieldName;
module.exports.ensureFieldNamesLoaded = ensureFieldNamesLoaded;
