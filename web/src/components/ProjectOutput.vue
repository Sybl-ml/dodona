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
        <b-tabs pills >
          <b-tab title="Select Model:" disabled></b-tab>
          <b-tab v-for="(data, index) in results" :key="index" variant="warning" :title="'Model '+(index+1)" active lazy >
            <br/>
            <b-table striped :items="parseData(data)" />
          </b-tab>
        </b-tabs>
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

export default {
  name: "ProjectOutput",
  props: {
    projectId: String,
    results: Array,
    loading: Boolean,
  },
  methods: {
    parseData(data) {
      console.log(data)
      return Papa.parse(data, { header: true }).data
    }
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
