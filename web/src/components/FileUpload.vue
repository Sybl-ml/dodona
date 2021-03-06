<template>
  <div>
    <b-collapse visible id="collapse-1">
      <b-form-file
        class="mb-3"
        placeholder="Choose a file or drop it here..."
        drop-placeholder="Drop file here..."
        @input="handleFile"
      />
    </b-collapse>
    <b-card-text>
      <b-button v-b-toggle.collapse-1.collapse-2 variant="mid" size="sm"
        >Advanced</b-button
      ></b-card-text
    >

    <b-collapse id="collapse-2" class="mt-2">
      <b-row>
        <b-col sm="6">
          <b-form-file
            class="mb-3"
            placeholder="Training Dataset..."
            drop-placeholder="Drop file here..."
            @input="handleTrain"
          />
        </b-col>
        <b-col sm="6">
          <b-form-file
            class="mb-3"
            placeholder="Precition Dataset..."
            drop-placeholder="Drop file here..."
            @input="handlePredict"
          />
        </b-col>
      </b-row>
    </b-collapse>
  </div>
</template>
<script>
export default {
  name: "FileUpload",
  data() {
    return {
      files: { train: null, predict: null },
    };
  },
  methods: {
    handleFile(e) {
      this.$emit("input", {file: e});
    },
    handleTrain(e) {
      this.files.train = e;
      this.$emit("input", this.files);
    },
    handlePredict(e) {
      this.files.predict = e;
      this.$emit("input", this.files);
    },
  },
};
</script>
