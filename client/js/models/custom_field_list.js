"use strict";

const api = require("../api.js");
const uri = require("../util/uri.js");
const AbstractList = require("./abstract_list.js");
const CustomField = require("./custom_field.js");

class CustomFieldList extends AbstractList {
    static get(fields) {
        return api
            .get(
                uri.formatApiLink("custom-fields", {
                    fields: fields.join(","),
                })
            )
            .then((response) => {
                return Promise.resolve(
                    Object.assign({}, response, {
                        results: CustomFieldList.fromResponse(response.results),
                    })
                );
            });
    }
}

CustomFieldList._itemClass = CustomField;
CustomFieldList._itemName = "custom_field";

module.exports = CustomFieldList;
