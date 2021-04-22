<template>
  <b-container fluid>
    <h4 class="mb-0">Analysis</h4>

    <b-form-group label="Select a Column:" label-for="dropdown-analysis-select">
      <b-form-select
        id="dropdown-analysis-select"
        size="sm"
        :options="getAnalysisOptions"
        v-model="analysis_selected"
      />
    </b-form-group>

    <b-row>
      <b-row v-if="this.analysis.columns[this.analysis_selected].Numerical">
        <data-analytics-bar
          :chart-data="
            this.analysis.columns[this.analysis_selected].Numerical.values
          "
          :name="this.analysis_selected"
          color="rgb(255, 99, 132)"
          ref="analysis_chart"
        />
        <p>
          MAX -
          {{ this.analysis.columns[this.analysis_selected].Numerical.max }}
        </p>
        <br/>
        <p>
          MIN -
          {{ this.analysis.columns[this.analysis_selected].Numerical.min }}
        </p>
        <br/>
        <p>
          AVG -
          {{ this.analysis.columns[this.analysis_selected].Numerical.avg }}
        </p>
      </b-row>
      <b-row v-else>
        <b-col lg="8">
        <data-analytics-bar
          :chart-data="
            this.analysis.columns[this.analysis_selected].Categorical.values
          "
          :name="this.analysis_selected"
          color="rgb(99, 255, 222)"
          ref="analysis_chart" 
        />
        </b-col>
        <b-col lg="4">
          <b-table hover striped :items="columns"></b-table>
        </b-col>
      </b-row>
    </b-row>
  </b-container>
</template>

<script>
import DataAnalyticsBar from "@/components/charts/DataAnalyticsBar";

export default {
  name: "ProjectAnalysis",
  components: {
    DataAnalyticsBar,
  },
  data() {
    return {
      analysis_selected: "",
    };
  },
  props: {
    id: String,
    analysis: Object,
  },
  computed: {
    getAnalysisOptions() {
      let keys = Object.keys(this.analysis.columns);
      this.analysis_selected = keys[0];
      return keys;
    },
    columns() {
      let columns_list = Object.keys(this.analysis.columns[this.analysis_selected].Categorical.values);
      let columns_obj = [];
      for (let col of columns_list) {
        columns_obj.push({attributes: col})
      }
      return columns_obj
    }
  },
};
</script>
