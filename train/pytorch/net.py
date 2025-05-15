
import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim
import random
import sys
import os
from multiprocessing import Pool

class Net(nn.Module):


    def __init__(self):
        super(Net, self).__init__()
        # 12 input image channels for each piecetype, 32 output channels, 3x3 square convolution
        # Pad after each convolution
        # kernel
        self.conv0 = nn.Conv2d(12, 32, 3, padding = (1, 1))
        self.conv1 = nn.Conv2d(32, 32, 3, padding = (1, 1))
        self.conv2 = nn.Conv2d(32, 32, 3, padding = (1, 1))
        self.conv3 = nn.Conv2d(32, 32, 3, padding = (1, 1))

        # an affine operation: y = Wx + b
        self.fc = nn.Linear(32 * 64, 64)  # 6*6 from image dimension

        # The two heads, one for wdl the other for moves remaining
        self.wdl_logSoftmax = nn.LogSoftmax(dim=1)
        self.wdl_head = nn.Linear(64, 3)
        self.moves_head = nn.Linear(64, 1)

        self.learning_rate = 0.1
        self.device = torch.device("cuda:0")
        self.wdl_criterion = nn.NLLLoss()
        self.moves_criterion = nn.MSELoss()
        self.optimizer = optim.SGD(self.parameters(), lr=self.learning_rate, momentum=0.9)

    def forward(self, x):
        #a = F.relu(self.conv0(x))
        #x = x + a
        #b = F.relu(self.conv1(x))
        #x = x + a + b
        #c = F.relu(self.conv2(x))
        #x = x + a + b + c
        #d = F.relu(self.conv3(x))
        #x = x + a + b + c + d
        x = F.relu(self.conv0(x))
        x = x + F.relu(self.conv1(x))
        x = x + F.relu(self.conv2(x))
        x = x + F.relu(self.conv3(x))

        x = x.view(-1, self.num_flat_features(x))
        x = F.relu(self.fc(x))

        # The two heads.  One is for the WDL percentage and one is for moves remaining.
        wdl = self.wdl_logSoftmax(self.wdl_head(x))
        moves_remaining = F.relu(self.moves_head(x.detach()))

        return wdl, moves_remaining

    def num_flat_features(self, x):
        size = x.size()[1:]  # all dimensions except the batch dimension
        num_features = 1
        for s in size:
            num_features *= s
        return num_features

    def train_file(self, inputs, wdl_labels, moves_labels):
        loss = 0.0
        running_loss = 0.0
        batch_size = 100
        stripe_count = int(len(inputs) / batch_size)
        for r in range(1):
            for i in range(stripe_count):
                # i is number of stripes in chunked list of minibatches
                t_inputs = torch.tensor(inputs[i::stripe_count], device=self.device)
                t_wdl_labels = torch.tensor(wdl_labels[i::stripe_count], device=self.device)
                t_moves_labels = torch.tensor(moves_labels[i::stripe_count], device=self.device)

                self.optimizer.zero_grad()
                wdl_outputs, moves_remaining_outputs = self(t_inputs)
                loss_wdl = self.wdl_criterion(wdl_outputs, t_wdl_labels)
                loss_moves = self.moves_criterion(moves_remaining_outputs, t_moves_labels)

                loss_wdl.backward()
                # loss_moves will only updates weights in its head since it was detached
                loss_moves.backward()
                self.optimizer.step()

                # print statistics
                running_loss += loss_wdl.item()

            # print every round
            if True:
                loss = running_loss/stripe_count
                print('\tRound {} Avg Loss: {}'.format(r, loss))
                running_loss = 0.0
        return loss

    def validation_loss(self, inputs, wdl_labels, moves_labels):
        loss = 0.0
        running_loss = 0.0
        batch_size = 100
        stripe_count = int(len(inputs) / batch_size)
        for r in range(1):
            for i in range(stripe_count):
                # i is number of stripes in chunked list of minibatches
                t_inputs = torch.tensor(inputs[i::stripe_count], device=self.device)
                t_wdl_labels = torch.tensor(wdl_labels[i::stripe_count], device=self.device)
                t_moves_labels = torch.tensor(moves_labels[i::stripe_count], device=self.device)

                wdl_outputs, moves_remaining_outputs = self(t_inputs)
                loss_wdl = self.wdl_criterion(wdl_outputs, t_wdl_labels)
                loss_moves = self.moves_criterion(moves_remaining_outputs, t_moves_labels)

                # print statistics
                running_loss += loss_wdl.item()

            # print every round
            if True:
                loss = running_loss/stripe_count
                print('Validation Data Avg Loss: {}'.format(loss))
                running_loss = 0.0
        return loss


def split(x):
    return x.strip().split(',')

def tensorfy(x):
    return board_to_tensor(x[0])

def create_label(x):
    return [float(x[2])]
