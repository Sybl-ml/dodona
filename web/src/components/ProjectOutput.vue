<template>
  <b-container fluid class="mt-3">
    <b-row>
      <b-col v-if="!results && !loading" class="text-center">
        <b-button @click="$emit('get-results')" variant="warning" class="px-5"
          >Load Results</b-button
        >
      </b-col>
      <b-col v-else-if="loading" class="text-center">
        <b-icon
          icon="arrow-counterclockwise"
          animation="spin-reverse"
          font-scale="4"
        ></b-icon>
      </b-col>
      <b-col v-else class="input-table">
        <br />
        <b-button
          @click="downloadCSVData()"
          variant="ready"
          class="px-5"
        >
          Download Predictions
        </b-button>
        <br />
        <br />
        {{ parsePredictions() }}
        <pagination-table
          :projectId="projectId"
          :dataset_type="'prediction'"
        />
      </b-col>
    </b-row>
  </b-container>
</template>

<style scoped>
.input-table {
  height: calc(50px * 12);
  overflow-y: scroll;
}
</style>

<script>
import Papa from "papaparse";
import VuetableFieldHandle from "vuetable-2/src/components/VuetableFieldHandle.vue";
import PaginationTable from "./PaginationTable.vue";

export default {
  name: "ProjectOutput",
  components: {
    PaginationTable,
  },
  data() {
    return {
      fields: null,
      pred_data: null,
      full_preds: "",
    };
  },
  props: {
    projectId: String,
    results: String,
    predict_data: String,
    datasetName: String,
    loading: Boolean,
  },
  methods: {
    parsePredictions() {
      this.full_preds = this.getFullPredictions(this.results)
      var parsed = Papa.parse(this.full_preds, { header: true });
      let new_fields = this.buildFields(parsed.meta.fields);
      this.fields = new_fields;
      this.pred_data = parsed.data;
    },
    getFullPredictions(data) {
      var new_data = [];
      var split_predict = this.predict_data.trim().split("\n");
      var split_predicted = data.split("\n");

      new_data.push(split_predict[0]);
      for (var i = 1; i < split_predict.length; i++) {
        var residual = split_predict[i].concat(split_predicted[i - 1]);
        new_data.push(residual);
      }

      let ret_data = new_data.join("\n").trim();

      return ret_data;
    },
    downloadCSVData() {
      const anchor = document.createElement("a");
      anchor.href = "data:text/csv;charset=utf-8," + encodeURIComponent(this.full_preds);
      anchor.target = "_blank";
      anchor.download = "predictions.csv";
      anchor.click();
    },
    buildFields(fields) {
      let built_fields = [
        {
          name: VuetableFieldHandle,
        },
      ];

      fields.forEach(function(item, index) {
        built_fields.push({
          name: item,
          title: item,
          sortField: item,
        });
      });
      return built_fields;
    },
  },
};
</script>
