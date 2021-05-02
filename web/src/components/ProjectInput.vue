<template>
  <b-container fluid>
    <b-row>
      <h4>{{ this.datasetName }}</h4>
    </b-row>
    <b-row>
      <b-col>
        <b-tabs class="mb-3" pills>
          <b-tab title="Select Dataset:" disabled></b-tab>
          <b-tab title="Training" active lazy>
            <br />
            <pagination-table :projectId="projectId" :dataset_type="'train'" />
          </b-tab>
          <b-tab title="Prediction" lazy>
            <br />
            <pagination-table
              :projectId="projectId"
              :dataset_type="'predict'"
            />
          </b-tab>
        </b-tabs>
      </b-col>
    </b-row>
  </b-container>
</template>

<script>
import PaginationTable from "./PaginationTable.vue";

export default {
  name: "ProjectInput",
  components: {
    PaginationTable,
  },
  props: {
    projectId: String,
    datasetName: String,
  },
  computed: {
    getDatasetDate() {
      return `${this.dataDate.toLocaleString("en-GB", {
        dateStyle: "short",
      })} - ${this.dataDate.toLocaleString("en-GB", {
        timeStyle: "short",
      })}`;
    },
  },
};
</script>
