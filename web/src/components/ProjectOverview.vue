<template>
  <b-container fluid>
    <b-container v-if="ready">
      <h3>Almost Done!</h3>
      <p>To start computation click the button below</p>
      <br>
      <p class="display-1 text-center">
        <b-link @click="start">
          <b-icon-play-fill variant="success"/>
        </b-link>
      </p>
    </b-container>
    <b-container v-else>
      <b-row>
        <b-col>
          <h4>&lt;Dataset Title&gt;</h4>
          <p>{{ getDatasetDate }}</p>
        </b-col>
      </b-row>
      <b-row class="input-table">
        <b-table striped :items="this.dataHead.data" />
      </b-row>
      <b-row>
        <b-col class="text-center" style="color: #4650e8">
          <b-link @click="$emit('input-tab')">See More...</b-link></b-col
        ></b-row
      >
    </b-container>
  </b-container>
</template>

<style scoped>
.input-table {
  overflow-y: scroll;
}
</style>

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
    ready: Boolean,
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
  methods: {
    async start() {
      let user_id = $cookies.get("token");
      try {
        await axios.post(
          `http://localhost:3001/api/projects/p/${this.projectId}/process`
        );
      } catch (err) {
        console.log(err);
      }
    },
  }
};
</script>
