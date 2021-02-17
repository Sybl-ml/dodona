<template>
  <b-container fluid class="mt-3">
    <b-row>
      <b-col v-if="!results && !loading" class="text-center">
        <b-button @click="getResults()" variant="warning" class="px-5"
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
            <b-button @click="downloadCSVData(data)" variant="ready" class="px-5">
              Download Predictions
            </b-button>
            <br/>
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
    datasetName: String,
  },
    data() {
    return {
      results: null,
      predict_data: "",
      loading: false
    };
  },

  methods: {
    async getResults(){
      this.loading = true;
      
      let project_predictions = await this.$http.get(
          `http://localhost:3001/api/projects/p/${this.projectId}/predictions`
        );

      this.results = project_predictions.data["predictions"];
      this.predict_data = project_predictions.data["predict_data"];
      this.loading = false;

    },
    parseData(data) {
      
      return Papa.parse(this.getFullPredictions(data), { header: true }).data
    },
    getFullPredictions(data) {

      var new_data = [];
      var split_predict = this.predict_data.split("\n");
      var split_predicted = data.split("\n");

      new_data.push(split_predict[0]);
      for (var i = 1; i < split_predict.length; i++) {
          var residual = split_predict[i].concat(split_predicted[i-1]);
          new_data.push(residual)
      }

      return new_data.join("\n");

    },

    downloadCSVData(data) {
      let csv = this.getFullPredictions(data)
  
      const anchor = document.createElement('a');
      anchor.href = 'data:text/csv;charset=utf-8,' + encodeURIComponent(csv);
      anchor.target = '_blank';
      anchor.download = "predictions_"+this.datasetName;
      anchor.click();
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
