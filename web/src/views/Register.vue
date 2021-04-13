<template>
  <b-container fluid class="d-flex flex-column flex-grow-1">
    <b-row class="justify-content-center text-center">
      <b-col lg="5" md="8" sm="12" xs="12">
        <icon-logo
          class="mt-5 mb-3"
          height="10em"
          width="10em"
          :show_text="false"
        />
        <h1 class="mb-3"><strong>Create A New Account</strong></h1>
        <b-card no-body class="text-left mt-3 mb-5 vh-80">
          <navigatable-tab
            :tabs="[
              { key: '1', title: '1. Name' },
              { key: '2', title: '2. Photo' },
              { key: '3', title: '3. Details' },
            ]"
          >
            <template v-slot:1>
              <h4>Personal Information</h4>

              <b-form-input
                class="mb-3"
                type="text"
                required
                placeholder="First Name"
                v-model="firstName"
              />
              <b-form-input
                class="mb-3"
                type="text"
                required
                placeholder="Last Name"
                v-model="lastName"
              />
              <b-form-checkbox
                class="mb-3"
                v-model="overAge"
              >I am over 18 (required)</b-form-checkbox>
            </template>

            <template v-slot:2>
              <avatar-upload @upload="onUpload" />
            </template>

            <template v-slot:3>
              <h4>Login Details</h4>
              <b-input-group class="mb-3" prepend="@">
                <b-form-input
                  type="email"
                  required
                  placeholder="Enter Email"
                  v-model="email"
                />
              </b-input-group>
              <b-input-group class="mb-3" prepend="#">
                <b-form-input
                  :type="passwordType"
                  required
                  placeholder="Password"
                  v-model="password"
                  pattern="^.{1,32}$"
                  title="Password must contain at least eight characters"
                />
                <template #append>
                  <b-input-group-text>
                    <b-icon
                      style="cursor: pointer"
                      :icon="passwordIcon"
                      @click="hidePassword = !hidePassword"
                    />
                  </b-input-group-text>
                </template>
              </b-input-group>
              <b-input-group class="mb-3" prepend="#">
                <b-form-input
                  :type="passwordType"
                  required
                  id="confirmPass"
                  placeholder="Confirm Password"
                  v-model="confirmPassword"
                /> </b-input-group
              ><b-tooltip
                v-if="confirmPassword != password"
                target="confirmPass"
                triggers="hover"
                variant="warning"
                >Passwords do not match
              </b-tooltip>

              <span id="submitButton" class="float-right">
                <b-button
                  size="sm"
                  variant="ready"
                  type="submit"
                  style="width: 10rem"
                  v-b-tooltip.hover
                  @click="onSubmit"
                  :disabled="!validCredentials"
                >
                  SIGN UP
                  <b-spinner v-show="submitted" small></b-spinner>
                  <b-icon-check-all
                    v-show="!submitted && validCredentials"
                  ></b-icon-check-all>
                </b-button> </span
              ><b-tooltip
                v-if="!validCredentials"
                target="submitButton"
                triggers="hover"
                placement="topleft"
                variant="danger"
              >
                Missing or Invalid Credentials
              </b-tooltip>
            </template>
          </navigatable-tab>
        </b-card>
      </b-col>
    </b-row>
    <b-row class="justify-content-center text-center">
      <b-alert v-model="failed" variant="danger" dismissible>
        Something is wrong with your infomation
      </b-alert> </b-row
    >
  </b-container>
</template>

<script>
import IconLogo from "@/components/icons/IconLogo";
import NavigatableTab from "@/components/NavigatableTab.vue";
import AvatarUpload from "@/components/AvatarUpload.vue";

export default {
  data() {
    return {
      email: "",
      password: "",
      confirmPassword: "",
      firstName: "",
      lastName: "",

      avatarSrc: null,

      submitted: false,
      hidePassword: true,
      failed: false,
      overAge: false,
    };
  },
  components: {
    IconLogo,
    NavigatableTab,
    AvatarUpload,
  },
  computed: {
    validCredentials() {
      return (
        this.email &&
        this.firstName &&
        this.lastName &&
        this.overAge &&
        this.password &&
        this.confirmPassword &&
        this.password === this.confirmPassword
      );
    },
    passwordType() {
      return this.hidePassword ? "password" : "text";
    },
    passwordIcon() {
      return this.hidePassword ? "eye-fill" : "eye-slash-fill";
    },
  },
  methods: {
    sleep(ms) {
      return new Promise((resolve) => {
        setTimeout(resolve, ms);
      });
    },
    async onSubmit() {
      this.submitted = true;

      try {
        let response = await this.$store.dispatch("register", {
          email: this.email,
          password: this.password,
          firstName: this.firstName,
          lastName: this.lastName,
        });

        $cookies.set("token", response.data.token, {
          path: "/",
          sameSite: true,
        });
        this.uploadAvatar();
        this.$router.push("dashboard");
      } catch (err) {
        this.failed = true;
      }

      await this.sleep(1000);
      this.submitted = false;
    },
    onUpload(avatarSrc) {
      this.avatarSrc = avatarSrc;
    },
    uploadAvatar() {
      if (this.avatarSrc) {
        this.$store.dispatch("uploadAvatar", this.avatarSrc.split(",")[1]);
      }
    },
  },
};
</script>
