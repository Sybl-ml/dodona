<template>
  <b-container fluid class="mt-3">
    <b-row>
      <b-col v-if="!data && !loading" class="text-center">
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
      <b-col v-else class="input-table">
        <b-table striped :items="this.data.data" />
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
import axios from "axios";
import Papa from "papaparse";

export default {
  name: "ProjectInput",
  props: {
    projectId: String,
    data: Object,
    loading: Boolean,
  },
  methods: {},
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
