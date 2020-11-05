<template>
  <b-container fluid>
    <h3>Look at all your data</h3>
    <b-row>
      <b-col v-if="loading" class="text-center">
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
  async created() {
    this.$emit("getData");
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
