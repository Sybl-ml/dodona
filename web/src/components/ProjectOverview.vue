<template>
  <b-container fluid>
    <b-row>
      <b-col lg="8" v-if="ready">
        <h4>Description:</h4>
        <p>{{ description }}</p>
        <h4>Linked Dataset:</h4>
        <b-button-group size="sm" class="mb-3">
          <b-button variant="secondary" @click="$emit('input-tab')">{{
            datasetName
          }}</b-button>
          <b-button variant="outline-secondary" v-b-modal.deleteDataCheck
            >X</b-button
          >
        </b-button-group>

        <b-modal
          id="deleteDataCheck"
          ref="deleteDataCheck"
          title="Are your sure?"
          hide-footer
        >
          <p>You are removing {{ datasetName }} from this project</p>
          <p>Please confirm you are happy to continue</p>
          <b-row class="justify-content-center text-center">
            <b-button class="m-2" variant="success" @click="deleteDataset"
              >Confirm</b-button
            >
            <b-button
              class="m-2"
              variant="warning"
              @click="$bvModal.hide('deleteDataCheck')"
              >Cancel</b-button
            >
          </b-row>
        </b-modal>

        <h4>Job Configuration:</h4>
        <b-progress
          :value="value"
          :variant="progressColor"
          height="2rem"
          show-progress
          animated
        ></b-progress>
        <b-container fluid>
          <b-row class="mt-4">
            <b-col>
              <b-form-group
                label="Timeout (mins)"
                label-for="dropdown-form-timeout"
              >
                <b-form-input
                  id="dropdown-form-timeout"
                  size="sm"
                  type="number"
                  v-model="timeout"
                ></b-form-input>
              </b-form-group>
            </b-col>
          </b-row>
          <b-row>
            <b-col>
              <b-form-group label="Problem Type" label-for="dropdown-form-type">
                <b-form-select
                  id="dropdown-form-type"
                  size="sm"
                  :options="problemTypeOptions"
                  v-model="problemType"
                />
              </b-form-group>
            </b-col>
          </b-row>
          <b-row>
            <b-col>
              <b-form-group
                label="Prediction Column"
                label-for="dropdown-pred-col"
              >
                <b-form-select
                  id="dropdown-pred-col"
                  size="sm"
                  :options="getColumnNames"
                  v-model="predColumn"
                />
              </b-form-group>
            </b-col>
          </b-row>
        </b-container>
        <h4>To start computation click the button below</h4>
        <p class="display-1 text-center">
          <b-button
            @click="start"
            :disabled="startDisabled"
            class="empty-button"
          >
            <b-icon-play-fill font-scale="7.5" variant="success" />
          </b-button>
        </p>
      </b-col>

      <b-col lg="8" v-else>
        <h4>Description:</h4>
        <p>{{ description }}</p>
        <h5>To continue you must provide a dataset</h5>
        <b-form-file
          class="mb-3"
          placeholder="Select a dataset"
          drop-placeholder="Drop file here..."
          v-model="file"
        />
        <b-button variant="secondary" @click="addData">Upload</b-button>
      </b-col>
    </b-row>
  </b-container>
</template>

<style scoped>
.input-table {
  overflow-y: scroll;
}

.empty-button {
  background-color: white !important;
  border-color: white !important;
}
</style>

<script>
export default {
  name: "ProjectOverview",
  data() {
    return {
      timeout: 10,
      value: 64,
      file: null,
      problemType: null,
      predColumn: null,

      problemTypeOptions: [
        {
          value: null,
          text: "Please Choose Classification or Regression",
        },
        {
          value: "classification",
          text: "Classification",
        },
        {
          value: "regression",
          text: "Regression",
        },
      ],
    };
  },
  props: {
    projectId: String,
    description: String,
    datasetName: String,
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
    progressColor() {
      if (this.value === 100) {
        return "completed";
      } else if (this.value < 25) {
        return "warning";
      } else if (this.value < 50) {
        return "primary";
      } else if (this.ready < 75) {
        return "ready";
      }
    },
    getColumnNames() {
      let keys = Object.keys(this.dataTypes);
      let options = [
        {
          value: null,
          text: "Please select the column to predict",
        },
      ];
      keys.forEach((key) => options.push({ value: key, text: key }));
      return options;
    },
    startDisabled() {
      return this.predColumn == null || this.problemType == null;
    },
  },
  methods: {
    async start() {
      try {
        await this.$http.post(
          `http://localhost:3001/api/projects/p/${this.projectId}/process`,
          {
            timeout: this.timeout,
            problemType: this.problemType,
            predictionColumn: this.predColumn,
          }
        );
      } catch (err) {
        console.log(err);
      }

      // this.$router.replace("/dashboard/"+this.projectId);
      this.$emit("update:project", this.projectId);
    },
    async deleteDataset() {
      try {
        let project_response = await this.$http.delete(
          `http://localhost:3001/api/projects/p/${this.projectId}/data`
        );
      } catch (err) {
        console.log(err);
      }
      this.$refs["deleteDataCheck"].hide();
    },
    addData() {
      if (this.file) {
        this.file_reader = new FileReader();
        this.file_reader.onload = this.sendFile;
        this.file_reader.readAsText(this.file);
      }
    },
    async sendFile(e) {
      let project_response = await this.$http.put(
        `http://localhost:3001/api/projects/p/${this.projectId}/data`,
        {
          name: this.file.name,
          content: e.target.result,
        }
      );

      // On Success should update dashboard using emitters
    },
  },
};
</script>
