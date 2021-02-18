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
            <b-button @click="downloadCSVData(data)" variant="ready" class="px-5">
              Download Predictions
            </b-button>
            <br/>
            <br/>
              {{parsePredictions(data)}}
              <my-vue-table
              :fields="fields"
              :display_data="pred_data"
              ></my-vue-table>

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
import VuetableFieldHandle from 'vuetable-2/src/components/VuetableFieldHandle.vue';
import MyVueTable from "./MyVueTable.vue";

export default {
  name: "ProjectOutput",
  components: {
    MyVueTable
  },
  data() {
    return {
      fields: null,
      pred_data: null,
      perPage: 1,
    };
  },
  props: {
    projectId: String,
    results: Array,
    predict_data: String,
    datasetName: String,
    loading: Boolean,
  },
  methods: {

    parsePredictions(data) {
      var parsed = Papa.parse(this.getFullPredictions(data), { header: true });
      let new_fields = this.buildFields(parsed.meta.fields);
      console.log(new_fields);
      this.fields = new_fields
      this.pred_data = parsed.data;

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
    },
    buildFields(fields) {
      let built_fields = [{
          name: VuetableFieldHandle
        }]
      
      fields.forEach(function (item, index) {
        built_fields.push({
            name: item,
            title: `<span class="orange glyphicon glyphicon-user"></span> ${item}`,
            sortField: item
          });
      });
      return built_fields;
    }
  },
};
</script>
