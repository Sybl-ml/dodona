<template>
  <b-container fluid>
    <b-row>
      <b-col>
        <h4>&lt;Dataset Title&gt;</h4>
        <p>{{ getDatasetDate }}</p>
      </b-col>
    </b-row>
    <b-table striped :items="this.dataHead.data" />
    <b-row>
      <b-col class="text-center" style="color: #4650e8">
        <b-link @click="$emit('input-tab')">See More...</b-link></b-col
      ></b-row
    >
  </b-container>
</template>

<script>
import axios from "axios";
import Papa from "papaparse";

export default {
  name: "ProjectOverview",
  props: {
    projectId: String,
    dataDate: Date,
    dataHead: Object,
    dataTypes: Object,
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
