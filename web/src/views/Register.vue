<template>
  <b-container fluid class="d-flex flex-column flex-grow-1">
    <b-row class="justify-content-center text-center">
      <b-col lg="3" md="6" sm="8" xs="12">
        <icon-logo
          class="mt-5 mb-3"
          height="10em"
          width="10em"
          :show_text="false"
        />
        <h1 class="mb-3"><strong>Create A New Account</strong></h1>
        <b-card bordered-variant="primary" class="text-center mt-3 mb-5">
          <b-form class="mt-3 mb-3" @submit.prevent="onSubmit">
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
                pattern="^.{8,32}$"
                title="Password must contain at least eight characters"
              />
              <template #append>
                <b-input-group-text>
                  <b-icon
                    style="cursor: pointer;"
                    :icon="passwordIcon"
                    @click="hidePassword = !hidePassword"
                  />
                </b-input-group-text>
              </template>
            </b-input-group>

            <b-input-group class="mb-3" prepend="#">
              <b-form-input
                type="password"
                required
                placeholder="Confirm Password"
                v-model="confirmPassword"
              />
            </b-input-group>

            <b-form-checkbox value="me" class="mb-3" v-model="overAge"
              >I am Over 13</b-form-checkbox
            >
            <b-button
              variant="primary"
              type="submit"
              block
              :disabled="!validCredentials"
            >
              SIGN UP
              <b-spinner v-show="submitted" small></b-spinner>
            </b-button>
          </b-form>
        </b-card>
      </b-col>
    </b-row>
    <b-row class="justify-content-center text-center">
      <b-alert v-model="failed" variant="danger" dismissible>
        Something is wrong with your infomation
      </b-alert>
    </b-row>
  </b-container>
</template>

<script>
import IconLogo from "@/components/icons/IconLogo";
import axios from "axios";
export default {
  data() {
    return {
      email: "",
      password: "",
      confirmPassword: "",
      overAge: false,
      firstName: "",
      lastName: "",

      submitted: false,
      hidePassword: true,
      failed: false,
    };
  },
  components: {
    IconLogo,
  },
  computed: {
    validCredentials() {
      return (
        this.email &&
        this.firstName &&
        this.lastName &&
        this.password &&
        this.confirmPassword &&
        this.overAge &&
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
      let response = await axios.post("http://localhost:3001/api/users/new", {
        email: this.email,
        password: this.password,
        firstName: this.firstName,
        lastName: this.lastName,
      });

      if (response) {
        response = response.data;
        if (response.token === "null") {
          this.failed = false;
        } else {
          $cookies.set("token", response.token, { path: "/", sameSite: true });
          this.$router.push("dashboard");
        }
      }

      await this.sleep(1000);
      
      this.failed = true;
      this.submitted = false;
    },
  },
};
</script>
