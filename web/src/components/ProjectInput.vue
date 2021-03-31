<template>
  <b-container fluid>
    <!-- <b-row>
      <h4>{{ this.datasetName }}</h4>
    </b-row> -->
    <b-row>
      <b-col v-if="!training_data && !loading" class="text-center">
        <b-row class="head-input-table">
          <b-table hover striped :items="this.dataHead.data" />
        </b-row>
        <b-button @click="$emit('get-data')" variant="primary" class="px-5"
          >Load Data</b-button
        >
      </b-col>
      <b-col v-else-if="loading" class="text-center">
        <b-icon
          icon="arrow-counterclockwise"
          animation="spin-reverse"
          font-scale="4"
        ></b-icon>
      </b-col>
      <b-col v-else>
        <b-tabs class="mb-3" pills>
          <b-tab title="Select Datset:" disabled></b-tab>
          <b-tab title="Training" active lazy>
            <br>
            <pagination-table
              v-if="this.training_data"
              :fields="buildFields(this.training_data.meta.fields)"
              :data="this.training_data.data"
            />
          </b-tab>
          <b-tab title="Prediction" active lazy>
            <br>
            <pagination-table
              v-if="this.predict_data"
              :fields="buildFields(this.predict_data.meta.fields)"
              :data="this.predict_data.data"
            />
          </b-tab>
        </b-tabs>
      </b-col>
    </b-row>
  </b-container>
</template>


<script>
import VuetableFieldHandle from "vuetable-2/src/components/VuetableFieldHandle.vue";
import PaginationTable from "./PaginationTable.vue";

export default {
  name: "ProjectInput",
  components: {
    PaginationTable,
  },
  props: {
    projectId: String,
    datasetName: String,
    training_data: Object,
    predict_data: Object,
    dataHead: Object,
    loading: Boolean,
  },
  methods: {
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
