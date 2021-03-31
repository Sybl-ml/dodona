<template>
  <b-container fluid>
    <b-card style="height:7rem;border:none;box-shadow:none">
      <h2>{{ name }}</h2>
      <p>
        {{ getProjectDate }}
        <b-badge
          pill
          variant="success"
          class="mx-1"
          v-for="tag in tags"
          v-bind:key="tag.id"
          >{{ tag }}</b-badge
        >
      </p>
    </b-card>
    <b-card no-body class="shadow">
      <b-tabs pills card>
        <b-tab title="Overview" active lazy ref="overviewTab">
          <project-overview
            :projectId="projectId"
            :description="description"
            :datasetName="datasetName"
            :key="projectId"
            :dataHead="dataHead"
            :dataDate="datasetDate"
            :dataTypes="dataTypes"
            :analysis="analysis"
            :analysis_loaded="analysis_loaded"
            :status="status"
            @update:project="updateProject"
            v-on:input-tab="viewInput"
          />
        </b-tab>
        <b-tab title="Input" ref="inputTab" :disabled="datasetName == '' ">
          <project-input
            :projectId="projectId"
            :key="projectId"
            :dataHead="dataHead"
            @get-data="fetchData"
            :training_data="training_data"
            :predict_data="predict_data"
            :datasetName="datasetName"
            :loading="loading"
          />
        </b-tab>
        <b-tab title="Output" lazy :disabled="false">
          <project-output
            :disabled="!results"
            :projectId="projectId"
            :key="projectId"
            @get-results="fetchResults"
            :results="results"
            :predict_data="predict_data"
            :datasetName="datasetName"
            :loading="results_loading"
          />
        </b-tab>
        <b-tab title="Settings" lazy>
          <project-settings
            :projectId="projectId"
            :key="projectId"
            :name="name"
            :description="description"
            :tags="tags"
            @update:name="updateName"
            @update:description="updateDescription"
            @update:tags="updateTags"
            @delete:project="$emit('delete:project', projectId)"
          />
        </b-tab>
      </b-tabs>
    </b-card>
  </b-container>
</template>

<script>
import Papa from "papaparse";
import ProjectOverview from "@/components/ProjectOverview";
import ProjectInput from "@/components/ProjectInput";
import ProjectOutput from "@/components/ProjectOutput";
import ProjectSettings from "@/components/ProjectSettings";

export default {
  name: "ProjectView",
  data() {
    return {
      name: "",
      description: "",
      status: "",
      tags: null,
      dateCreated: new Date(),

      datasetDate: new Date(),
      datasetName: "",
      dataHead: {},
      dataTypes: {},

      training_data: null,
      predict_data: null,
      loading: false,

      analysis: {},
      analysis_loaded: false,

      results: null,
      predict_data: null,
      results_loading: false,
    };
  },
  props: {
    show: Boolean,
    projectId: String,
  },
  components: {
    ProjectOverview,
    ProjectInput,
    ProjectOutput,
    ProjectSettings,
  },
  watch: {
    projectId: function() {
      this.resetProject();
      this.fetchProject();
      this.$refs.overviewTab.activate();
    },
  },
  async mounted() {
    this.fetchProject();
  },
  methods: {
    async fetchProject() {
      let project_response = await this.$http.get(
        `api/projects/${this.projectId}`
      );

      let project_details = project_response.data.details;
      let project_info = project_response.data.project;
      let project_analysis = project_response.data.analysis;

      this.name = project_info.name;
      this.description = project_info.description;
      this.tags = project_info.tags;
      this.dateCreated = new Date(project_info.date_created.$date);
      this.status = project_info.status;
      if (project_details) {
        this.dataHead = Papa.parse(project_details.head, { header: true });
        this.datasetName = project_details.dataset_name;
        this.datasetDate = new Date(project_details.date_created.$date);
        this.dataTypes = project_details.column_types;
      }
      if (project_analysis) {
        this.analysis = project_analysis;
        this.analysis_loaded = true;
      }
    },
    async fetchData() {
      this.loading = true;

      let train_response = await this.$http.get(
        `api/projects/${this.projectId}/data/train`
      );

      let train = train_response.data;

      this.training_data = Papa.parse(train, { header: true });

      let predict_response = await this.$http.get(
        `api/projects/${this.projectId}/data/predict`
      );

      let predict = predict_response.data;

      this.predict_data = Papa.parse(predict, { header: true });

      this.loading = false;
    },
    async fetchResults() {
      this.results_loading = true;

      let project_predict = await this.$http.get(
        `api/projects/${this.projectId}/data/predict`
      );


      let project_predictions = await this.$http.get(
        `api/projects/${this.projectId}/predictions`
      );

      this.results = project_predictions.data;
      this.predict_data = project_predict.data;
      this.results_loading = false;
    },
    resetProject() {
      // this.name = "";
      // this.description = "";
      // this.dateCreated = new Date();

      this.datasetName = "";
      this.datasetDate = new Date();
      this.dataHead = {};
      this.dataTypes = {};

      this.analysis = null;
      this.results = null;
      this.predict_data = null;
      this.training_data = null;
      
      this.prediction_data = null;
      this.loading = false;
      this.results_loading = false;
      this.analysis_loaded = false;
    },
    viewInput() {
      this.$refs.inputTab.activate();
      this.fetchData();
    },
    updateName(newName) {
      this.name = newName;
      this.$emit("update:name", newName, this.projectId);
    },
    updateDescription(newDescription) {
      this.description = newDescription;
      this.$emit("update:description", newDescription, this.projectId);
    },
    updateTags(newTags) {
      this.tags = newTags;
      this.$emit("update:tags", newTags, this.projectId);
    },
    updateProject(id) {
      this.$emit("update:project", id);
    },
  },
  computed: {
    getProjectDate() {
      if (!this.name) {
        return "";
      }
      return `${this.dateCreated.toLocaleString("en-GB", {
        dateStyle: "short",
      })} - ${this.dateCreated.toLocaleString("en-GB", {
        timeStyle: "short",
      })}`;
    },
  },
};
</script>
