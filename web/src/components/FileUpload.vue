<template>
  <b-row>
    <b-col sm="9">
      <b-collapse visible id="collapse-1">
        <b-form-file
          class="mb-3"
          placeholder="Choose a file or drop it here..."
          drop-placeholder="Drop file here..."
          @input="handleFile"
          v-b-tooltip.hover.top="'Upload a single dataset with complete rows (for training) and incomplete rows (for predictions)'"
        />
      </b-collapse>

      <b-collapse id="collapse-2">
        <b-row>
          <b-col sm="6">
            <b-form-file
              class="mb-3"
              placeholder="Training ..."
              drop-placeholder="Drop file here..."
              @input="handleTrain"
              v-b-tooltip.hover.top="'Data (with labels) used to train machine learning models'"
            />
          </b-col>
          <b-col sm="6">
            <b-form-file
              class="mb-3"
              placeholder="Prediction ..."
              drop-placeholder="Drop file here..."
              @input="handlePredict"
              v-b-tooltip.hover.top="'Data (without labels) which predictions are made on'"
            />
          </b-col>
        </b-row>
      </b-collapse>
    </b-col>
    <b-col sm="3">
      <b-button
        v-b-toggle.collapse-1.collapse-2
        variant="mid"
        block
        pill
        @click="changeState"
        >{{ advanced ? "Advanced" : "Simple" }}</b-button
      >
    </b-col>
  </b-row>
</template>
<script>
export default {
  name: "FileUpload",
  data() {
    return {
      files: { train: null, predict: null },
      advanced: true,
    };
  },
  methods: {
    handleFile(e) {
      this.$emit("input", { file: e });
    },
    handleTrain(e) {
      this.files.train = e;
      this.$emit("input", this.files);
    },
    handlePredict(e) {
      this.files.predict = e;
      this.$emit("input", this.files);
    },
    changeState(e) {
      this.advanced = !this.advanced;
    },
  },
};
</script>
