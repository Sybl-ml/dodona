<template>
  <b-container fluid class="mt-3">
    <b-row>
      <b-col>
        <b-button
          @click="downloadCSVData()"
          variant="ready"
          class="px-5"
        >
          Download Predictions
        </b-button>
        <br />
        <br />
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
    datasetName: String,
  },
  methods: {
    async downloadCSVData() {
      let page_data = await this.$http.get(
        `api/projects/${this.projectId}/data/prediction`
      );
      const anchor = document.createElement("a");
      anchor.href = "data:text/csv;charset=utf-8," + encodeURIComponent(page_data.data);
      anchor.target = "_blank";
      anchor.download = "predictions.csv";
      anchor.click();
    },
  },
};
</script>
