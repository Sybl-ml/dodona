<template>
  <b-container fluid>
    <b-card style="height:7rem;border:none;box-shadow:none">
      <h2>{{ name }}</h2>
      <p>{{ getProjectDate }}</p>
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
            :ready="status == 'Ready'"
            @update:project="updateProject"
            v-on:input-tab="viewInput"
          />
        </b-tab>
        <b-tab title="Input" ref="inputTab">
          <project-input
            :projectId="projectId"
            :key="projectId"
            :dataHead="dataHead"
            @get-data="fetchData"
            :data="data"
            :datasetName="datasetName"
            :loading="loading"
          />
        </b-tab>
        <b-tab title="Output" lazy>
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
            @update:name="updateName"
            @update:description="updateDescription"
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
      dateCreated: new Date(),

      datasetDate: new Date(),
      datasetName: "",
      dataHead: {},
      dataTypes: {},

      data: null,
      loading: false,

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
        `http://localhost:3001/api/projects/p/${this.projectId}`
      );

      let project_details = project_response.data.details;
      let project_info = project_response.data.project;

      this.name = project_info.name;
      this.description = project_info.description;
      this.dateCreated = new Date(project_info.date_created.$date);
      this.status = project_info.status;
      if (project_details) {
        this.dataHead = Papa.parse(project_details.head, { header: true });
        this.datasetName = project_details.dataset_name;
        this.datasetDate = new Date(project_details.date_created.$date);
        this.dataTypes = project_details.column_types;
      }
    },
    async fetchData() {
      this.loading = true;

      let project_response = await this.$http.get(
        `http://localhost:3001/api/projects/p/${this.projectId}/data`
      );

      let project_data = project_response.data.dataset;

      this.data = Papa.parse(project_data, { header: true });
      this.loading = false;
    },
    async fetchResults() {
      this.results_loading = true;

      let project_predictions = await this.$http.get(
        `http://localhost:3001/api/projects/p/${this.projectId}/predictions`
      );
      this.results = project_predictions.data["predictions"];
      this.predict_data = project_predictions.data["predict_data"];
      this.results_loading = false;
    },
    resetProject() {
      // this.name = "";
      // this.description = "";
      // this.dateCreated = new Date();

      // this.datasetName = "";
      // this.datasetDate = new Date();
      // this.dataHead = {};
      // this.dataTypes = {};

      this.results = null;
      this.predict_data = null;
      this.data = null;
      this.loading = false;
      this.results_loading = false;
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
