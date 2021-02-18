<template>
  <b-container fluid >
    <b-row>
      <h4>{{this.datasetName}}
      </h4>
    </b-row>
    <b-row>
      <b-col v-if="!data && !loading" class="text-center">
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
      <b-col v-else class="input-table">
        <b-table hover striped :items="this.training_data.data" />
      </b-col>
    </b-row>
  </b-container>
</template>

<style scoped>
.input-table {
  height: calc(50px * 12);
  overflow-y: scroll;
}
.head-input-table {
  height: calc(52px * 6);
}
</style>

<script>
export default {
  name: "ProjectInput",
  props: {
    projectId: String,
    datasetName: String,
    training_data: Object,
    dataHead: Object,
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
